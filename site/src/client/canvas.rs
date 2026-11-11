//! Le dessin des deux animations 2D (`crate::sim`) dans leur canvas. Elles ne tournent
//! que lorsqu'elles sont à l'écran ; avec « moins d'animations », elles montrent
//! directement leur état final.

use std::cell::{Cell, RefCell};
use std::f64::consts::{PI, TAU};
use std::rc::Rc;

use paquets::active::State;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d as Ctx, HtmlCanvasElement, IntersectionObserver,
    IntersectionObserverEntry, IntersectionObserverInit, PointerEvent,
};

use super::{reduced_motion, request_frame, window};
use crate::content::IDEAS;
use crate::islands::{BALLOT_IDEAS, Scene};
use crate::sim::{Ball, BallotScene, HeroScene};

enum Live {
    Hero(HeroScene),
    Ballot(BallotScene, State<usize>),
}

impl Live {
    fn size(&self) -> (f64, f64) {
        match self {
            Live::Hero(_) => (f64::from(HeroScene::WIDTH), f64::from(HeroScene::HEIGHT)),
            Live::Ballot(..) => (
                f64::from(BallotScene::WIDTH),
                f64::from(BallotScene::HEIGHT),
            ),
        }
    }

    fn update(&mut self, dt: f32) {
        match self {
            Live::Hero(scene) => scene.update(dt),
            Live::Ballot(scene, counter) => {
                scene.update(dt);
                if counter.is_alive() && counter.get() != scene.dropped {
                    counter.set(scene.dropped);
                }
            }
        }
    }

    fn draw(&self, ctx: &Ctx) {
        match self {
            Live::Hero(scene) => draw_hero(ctx, scene),
            Live::Ballot(scene, _) => draw_ballot(ctx, scene),
        }
    }
}

/// Ce qui arrête l'animation et l'observation quand l'îlot s'en va.
pub struct Visible {
    observer: Option<IntersectionObserver>,
    running: Rc<Cell<bool>>,
    canvas: HtmlCanvasElement,
    _visibility: Closure<dyn Fn(js_sys::Array)>,
    tap: Option<Closure<dyn Fn(PointerEvent)>>,
}

impl Drop for Visible {
    fn drop(&mut self) {
        self.running.set(false);
        if let Some(observer) = &self.observer {
            observer.disconnect();
        }
        if let Some(tap) = &self.tap {
            let _ = self
                .canvas
                .remove_event_listener_with_callback("pointerdown", tap.as_ref().unchecked_ref());
        }
    }
}

/// La transformation du monde logique vers le canvas (centré, à l'échelle, net en HiDPI).
/// Rend aussi l'échelle et le décalage, pour retrouver un point touché.
fn fit(canvas: &HtmlCanvasElement, ctx: &Ctx, (w, h): (f64, f64)) -> (f64, f64, f64) {
    let rect = canvas.get_bounding_client_rect();
    let dpr = window().device_pixel_ratio().min(2.0);
    let (cw, ch) = (rect.width().max(1.0), rect.height().max(1.0));
    let (pw, ph) = ((cw * dpr).round() as u32, (ch * dpr).round() as u32);
    if canvas.width() != pw || canvas.height() != ph {
        canvas.set_width(pw);
        canvas.set_height(ph);
    }
    let k = (cw / w).min(ch / h);
    let (ox, oy) = ((cw - w * k) / 2.0, (ch - h * k) / 2.0);
    let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    ctx.clear_rect(0.0, 0.0, f64::from(pw), f64::from(ph));
    let _ = ctx.set_transform(k * dpr, 0.0, 0.0, k * dpr, ox * dpr, oy * dpr);
    (k, ox, oy)
}

fn render(canvas: &HtmlCanvasElement, ctx: &Ctx, live: &Live) {
    fit(canvas, ctx, live.size());
    live.draw(ctx);
}

pub fn start_when_visible(canvas: &web_sys::Element, scene: Scene) -> Option<Visible> {
    let canvas: HtmlCanvasElement = canvas.clone().dyn_into().ok()?;
    let ctx: Ctx = canvas.get_context("2d").ok()??.dyn_into().ok()?;
    let live = Rc::new(RefCell::new(match scene {
        Scene::Hero => Live::Hero(HeroScene::new()),
        Scene::Ballot(counter) => Live::Ballot(BallotScene::new(), counter),
    }));
    let _ = canvas.class_list().add_1("is-live");

    // Moins d'animations : l'état final, dessiné une fois.
    if reduced_motion() {
        {
            let mut live = live.borrow_mut();
            for _ in 0..(12 * 60) {
                live.update(1.0 / 60.0);
            }
        }
        render(&canvas, &ctx, &live.borrow());
    }

    let running = Rc::new(Cell::new(false));
    let start = {
        let (canvas, ctx, live, running) =
            (canvas.clone(), ctx.clone(), live.clone(), running.clone());
        move || {
            if running.replace(true) || reduced_motion() {
                return;
            }
            // Le dépouillement recommence quand il revient à l'écran une fois fini.
            if let Live::Ballot(scene, counter) = &mut *live.borrow_mut()
                && scene.crown().is_some()
            {
                *scene = BallotScene::new();
                counter.set(0);
            }
            let last = Cell::new(None::<f64>);
            let (canvas, ctx, live, running) =
                (canvas.clone(), ctx.clone(), live.clone(), running.clone());
            request_frame(move |now| {
                if !running.get() {
                    return false;
                }
                let dt = last
                    .replace(Some(now))
                    .map_or(0.0, |t| ((now - t) / 1000.0) as f32);
                let mut live = live.borrow_mut();
                live.update(dt.min(0.05));
                render(&canvas, &ctx, &live);
                true
            });
        }
    };

    let visibility = {
        let running = running.clone();
        Closure::<dyn Fn(js_sys::Array)>::new(move |entries: js_sys::Array| {
            for entry in entries.iter() {
                let Ok(entry) = entry.dyn_into::<IntersectionObserverEntry>() else {
                    continue;
                };
                if entry.is_intersecting() {
                    start();
                } else {
                    running.set(false);
                }
            }
        })
    };
    let options = IntersectionObserverInit::new();
    options.set_root_margin("60px 0px");
    let observer =
        IntersectionObserver::new_with_options(visibility.as_ref().unchecked_ref(), &options).ok();
    if let Some(observer) = &observer {
        observer.observe(&canvas);
    }

    // Un appui sur l'accueil lâche un jeton là où il tombe.
    let tap = matches!(scene, Scene::Hero).then(|| {
        let (target, live) = (canvas.clone(), live.clone());
        let tap = Closure::<dyn Fn(PointerEvent)>::new(move |e: PointerEvent| {
            let rect = target.get_bounding_client_rect();
            let (w, h) = live.borrow().size();
            let k = (rect.width() / w).min(rect.height() / h);
            let ox = (rect.width() - w * k) / 2.0;
            let x = (f64::from(e.client_x()) - rect.left() - ox) / k;
            if let Live::Hero(scene) = &mut *live.borrow_mut() {
                scene.drop_at(x as f32);
                super::haptic();
            }
        });
        let _ =
            canvas.add_event_listener_with_callback("pointerdown", tap.as_ref().unchecked_ref());
        tap
    });

    Some(Visible {
        observer,
        running,
        canvas,
        _visibility: visibility,
        tap,
    })
}

// ----- Le dessin ---------------------------------------------------------------------------

fn hsl(hue: f64, s: u8, l: u8, a: f64) -> String {
    format!("hsl({hue:.0} {s}% {l}% / {a})")
}

fn rounded(ctx: &Ctx, x: f64, y: f64, w: f64, h: f64, r: f64) {
    ctx.begin_path();
    ctx.move_to(x + r, y);
    let _ = ctx.arc_to(x + w, y, x + w, y + h, r);
    let _ = ctx.arc_to(x + w, y + h, x, y + h, r);
    let _ = ctx.arc_to(x, y + h, x, y, r);
    let _ = ctx.arc_to(x, y, x + w, y, r);
    ctx.close_path();
}

fn glow(ctx: &Ctx, x: f64, y: f64, r: f64, color: &str) {
    let g = ctx.create_radial_gradient(x, y, 0.0, x, y, r);
    if let Ok(g) = g {
        let _ = g.add_color_stop(0.0, color);
        let _ = g.add_color_stop(1.0, "rgba(13,10,31,0)");
        ctx.set_fill_style_canvas_gradient(&g);
        ctx.fill_rect(x - r, y - r, 2.0 * r, 2.0 * r);
    }
}

fn draw_hero(ctx: &Ctx, scene: &HeroScene) {
    let t = f64::from(scene.time);
    // Les halos qui respirent.
    glow(
        ctx,
        200.0,
        330.0,
        190.0 + 10.0 * (t * 1.2).sin(),
        "rgba(166,107,255,0.35)",
    );
    glow(ctx, 90.0, 120.0, 120.0, "rgba(255,107,107,0.16)");
    glow(ctx, 320.0, 110.0, 120.0, "rgba(62,224,208,0.14)");

    // Les cartes d'idées.
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    for (i, idea) in IDEAS.iter().enumerate() {
        let (x, y, a) = scene.card(i);
        let hue = f64::from(idea.hue);
        ctx.save();
        let _ = ctx.translate(f64::from(x), f64::from(y));
        let _ = ctx.rotate(f64::from(a));
        ctx.set_shadow_color(&hsl(hue, 80, 40, 0.55));
        ctx.set_shadow_blur(22.0);
        ctx.set_shadow_offset_y(10.0);
        let g = ctx.create_linear_gradient(-25.0, -36.0, 25.0, 36.0);
        let _ = g.add_color_stop(0.0, &hsl(hue, 85, 66, 1.0));
        let _ = g.add_color_stop(1.0, &hsl(hue + 35.0, 70, 42, 1.0));
        ctx.set_fill_style_canvas_gradient(&g);
        rounded(ctx, -25.0, -36.0, 50.0, 72.0, 10.0);
        ctx.fill();
        ctx.set_shadow_color("transparent");
        ctx.set_fill_style_str("rgba(255,255,255,0.22)");
        rounded(ctx, -25.0, -36.0, 50.0, 26.0, 10.0);
        ctx.fill();
        ctx.set_font("26px system-ui, 'Apple Color Emoji', 'Segoe UI Emoji', sans-serif");
        ctx.set_fill_style_str("#fff");
        let _ = ctx.fill_text(idea.emoji, 0.0, 4.0);
        ctx.restore();
    }

    // Le plateau.
    ctx.save();
    ctx.set_line_width(9.0);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    let g = ctx.create_linear_gradient(
        f64::from(HeroScene::TRAY_LEFT),
        0.0,
        f64::from(HeroScene::TRAY_RIGHT),
        0.0,
    );
    let _ = g.add_color_stop(0.0, "#ff6b6b");
    let _ = g.add_color_stop(0.5, "#a66bff");
    let _ = g.add_color_stop(1.0, "#3ee0d0");
    ctx.set_stroke_style_canvas_gradient(&g);
    ctx.set_shadow_color("rgba(166,107,255,0.8)");
    ctx.set_shadow_blur(18.0);
    ctx.begin_path();
    let (l, r, f, top) = (
        f64::from(HeroScene::TRAY_LEFT),
        f64::from(HeroScene::TRAY_RIGHT),
        f64::from(HeroScene::TRAY_FLOOR),
        f64::from(HeroScene::TRAY_TOP),
    );
    ctx.move_to(l, top);
    ctx.line_to(l + 12.0, f);
    ctx.line_to(r - 12.0, f);
    ctx.line_to(r, top);
    ctx.stroke();
    ctx.restore();

    // Les jetons.
    for ball in &scene.world.balls {
        draw_token(ctx, ball);
    }
}

fn draw_token(ctx: &Ctx, b: &Ball) {
    let (x, y, r) = (f64::from(b.x), f64::from(b.y), f64::from(b.r));
    let (light, dark) = if b.yes {
        ("#7ff5c4", "#22b07a")
    } else {
        ("#ff9fbe", "#d93d72")
    };
    ctx.save();
    let _ = ctx.translate(x, y);
    if let Ok(g) = ctx.create_radial_gradient(-r * 0.4, -r * 0.4, 1.0, 0.0, 0.0, r) {
        let _ = g.add_color_stop(0.0, light);
        let _ = g.add_color_stop(1.0, dark);
        ctx.set_fill_style_canvas_gradient(&g);
    }
    ctx.begin_path();
    let _ = ctx.arc(0.0, 0.0, r, 0.0, TAU);
    ctx.fill();
    let _ = ctx.rotate(f64::from(b.angle));
    ctx.set_stroke_style_str("rgba(255,255,255,0.95)");
    ctx.set_line_width(3.2);
    ctx.set_line_cap("round");
    ctx.begin_path();
    if b.yes {
        ctx.move_to(-r * 0.42, 0.0);
        ctx.line_to(-r * 0.1, r * 0.32);
        ctx.line_to(r * 0.45, -r * 0.3);
    } else {
        ctx.move_to(-r * 0.35, -r * 0.35);
        ctx.line_to(r * 0.35, r * 0.35);
        ctx.move_to(r * 0.35, -r * 0.35);
        ctx.line_to(-r * 0.35, r * 0.35);
    }
    ctx.stroke();
    ctx.restore();
}

fn draw_ballot(ctx: &Ctx, scene: &BallotScene) {
    let bins = crate::sim::VOTES.len();
    let (left, bin, top, floor) = (
        f64::from(BallotScene::LEFT),
        f64::from(BallotScene::BIN),
        f64::from(BallotScene::TOP),
        f64::from(BallotScene::FLOOR),
    );
    let winner = BallotScene::winner();
    let crown = scene.crown();
    // Le verre des urnes, teinté de leur idée ; la gagnante s'illumine.
    for i in 0..bins {
        let hue = f64::from(IDEAS[BALLOT_IDEAS[i]].hue);
        let lit = crown.is_some() && i == winner;
        ctx.set_fill_style_str(&hsl(hue, 80, 60, if lit { 0.22 } else { 0.08 }));
        ctx.fill_rect(left + i as f64 * bin + 3.0, top, bin - 6.0, floor - top);
        if lit {
            glow(
                ctx,
                left + (i as f64 + 0.5) * bin,
                top + 40.0,
                90.0,
                "rgba(255,200,61,0.25)",
            );
        }
    }
    // Les cloisons et le fond.
    ctx.save();
    ctx.set_stroke_style_str("#a66bff");
    ctx.set_shadow_color("rgba(166,107,255,0.7)");
    ctx.set_shadow_blur(10.0);
    ctx.set_line_width(4.0);
    ctx.set_line_cap("round");
    ctx.begin_path();
    for i in 0..=bins {
        let x = left + i as f64 * bin;
        ctx.move_to(x, top);
        ctx.line_to(x, floor);
    }
    ctx.move_to(left, floor);
    ctx.line_to(left + bins as f64 * bin, floor);
    ctx.stroke();
    ctx.restore();

    // Les billes.
    for b in &scene.world.balls {
        let hue = f64::from(IDEAS[BALLOT_IDEAS[b.kind]].hue);
        let (x, y, r) = (f64::from(b.x), f64::from(b.y), f64::from(b.r));
        if let Ok(g) = ctx.create_radial_gradient(x - r * 0.4, y - r * 0.4, 1.0, x, y, r) {
            let _ = g.add_color_stop(0.0, &hsl(hue, 90, 78, 1.0));
            let _ = g.add_color_stop(1.0, &hsl(hue + 20.0, 75, 45, 1.0));
            ctx.set_fill_style_canvas_gradient(&g);
        }
        ctx.begin_path();
        let _ = ctx.arc(x, y, r, 0.0, TAU);
        ctx.fill();
    }

    // La couronne du gagnant.
    if let Some(cy) = crown {
        let (cx, cy) = (BallotScene::center(winner) as f64, f64::from(cy));
        ctx.save();
        let _ = ctx.translate(cx, cy);
        let _ = ctx.rotate(0.08 * (f64::from(scene.time) * 2.0).sin());
        ctx.set_shadow_color("rgba(255,200,61,0.9)");
        ctx.set_shadow_blur(16.0);
        ctx.set_fill_style_str("#ffc83d");
        ctx.begin_path();
        ctx.move_to(-22.0, 12.0);
        ctx.line_to(-24.0, -10.0);
        ctx.line_to(-11.0, 0.0);
        ctx.line_to(0.0, -16.0);
        ctx.line_to(11.0, 0.0);
        ctx.line_to(24.0, -10.0);
        ctx.line_to(22.0, 12.0);
        ctx.close_path();
        ctx.fill();
        ctx.set_fill_style_str("#fff6d6");
        for (x, y) in [(-24.0, -10.0), (0.0, -16.0), (24.0, -10.0)] {
            ctx.begin_path();
            let _ = ctx.arc(x, y, 3.0, 0.0, PI * 2.0);
            ctx.fill();
        }
        ctx.restore();
    }
}
