//! Les parties interactives du site. Chaque îlot est rendu par le serveur comme une vue
//! ordinaire puis, compilé en WebAssembly, prend vie dans le navigateur depuis le même code.

use paquets::prelude::*;

use crate::content::{FRIENDS, IDEAS, leaders, tally, winner};
use crate::icons::icon;

/// Chaque îlot, sous le nom donné à [`island`].
pub const ISLANDS: [(&str, paquets::active::IslandFn); 4] = [
    ("hero", hero),
    ("ballot", ballot),
    ("swipe", swipe),
    ("flap", flap),
];

// ----- Les animations 2D -------------------------------------------------------------------

/// La scène d'accueil : le serveur rend le canvas, le navigateur l'anime en 2D quand il
/// est visible et s'arrête quand il sort de l'écran.
pub fn hero(_props: &str) -> Node {
    canvas()
        .class("scene scene--hero")
        .attr("role", "img")
        .attr(
            "aria-label",
            "Six cartes d'idées flottent au-dessus d'un plateau, des jetons oui et non tombent et s'empilent",
        )
        .use_action(|canvas| start_scene(canvas, Scene::Hero))
        .into()
}

/// L'idée de chaque urne du dépouillement, dans [`IDEAS`].
pub const BALLOT_IDEAS: [usize; 4] = [0, 1, 2, 3];

/// Le dépouillement en urnes, avec le compteur de billes tombées.
pub fn ballot(_props: &str) -> Node {
    use crate::sim::{BallotScene, VOTES};
    let dropped = use_state(0usize);
    let total: usize = VOTES.iter().sum();
    let best = BallotScene::winner();
    div()
        .class("ballot")
        .child(
            canvas()
                .class("scene scene--ballot")
                .attr("role", "img")
                .attr(
                    "aria-label",
                    "Des billes de vote tombent dans quatre urnes ; la plus remplie reçoit une couronne",
                )
                .use_action(move |canvas| start_scene(canvas, Scene::Ballot(dropped))),
        )
        .child(
            ol().class("ballot__labels")
                .children(BALLOT_IDEAS.iter().enumerate().map(|(bin, &idea)| {
                    li().class_if("is-winner", move || best == bin && dropped.get() >= total)
                        .style(format!("--hue:{}", IDEAS[idea].hue))
                        .child(span().class("ballot__emoji").text(IDEAS[idea].emoji))
                        .child(strong().text(VOTES[bin].to_string()))
                        .child(small().text(IDEAS[idea].kind))
                })),
        )
        .child(
            p().class("ballot__status")
                .attr("aria-live", "polite")
                .text_dyn(move || {
                    let n = dropped.get();
                    let idea = IDEAS[BALLOT_IDEAS[best]];
                    if n == 0 {
                        format!("{total} votes, quatre idées.")
                    } else if n < total {
                        format!("Dépouillement… {n} / {total}")
                    } else {
                        format!(
                            "{} {} l'emporte avec {} oui.",
                            idea.emoji, idea.title, VOTES[best]
                        )
                    }
                }),
        )
        .into()
}

/// Quelle scène animer.
#[derive(Clone, Copy)]
pub enum Scene {
    Hero,
    Ballot(State<usize>),
}

#[cfg(target_arch = "wasm32")]
fn start_scene(canvas: &paquets::active::Dom, scene: Scene) -> impl FnOnce() + use<> {
    let running = crate::client::canvas::start_when_visible(canvas, scene);
    move || drop(running)
}

#[cfg(not(target_arch = "wasm32"))]
fn start_scene(_canvas: &paquets::active::Dom, _scene: Scene) -> impl FnOnce() + use<> {
    || {}
}

// ----- La démo de vote -------------------------------------------------------------------

/// Le déplacement au-delà duquel une carte lâchée est un vote (pixels).
pub const THRESHOLD: f64 = 90.0;

/// La transformation d'une carte, selon sa place dans la pile et le glissement en cours.
pub fn card_transform(offset: isize, vote: Option<bool>, dx: f64, dy: f64) -> String {
    match offset {
        // Déjà votée : elle s'envole du côté de son vote.
        o if o < 0 => {
            let side = if vote == Some(true) { 1.0 } else { -1.0 };
            format!(
                "translate({}%, -8%) rotate({}deg)",
                side * 160.0,
                side * 28.0
            )
        }
        // Celle du dessus suit le doigt.
        0 => format!(
            "translate({dx:.1}px, {:.1}px) rotate({:.2}deg)",
            dy * 0.35,
            dx * 0.06
        ),
        // Les suivantes attendent, en retrait.
        o => {
            let o = o.min(3) as f64;
            format!(
                "translate(0, {:.0}px) scale({:.3})",
                o * 14.0,
                1.0 - o * 0.05
            )
        }
    }
}

/// Une pile de cartes à glisser : à droite oui, à gauche non. Les amis ont déjà voté ;
/// à la fin, le classement, et le tirage au sort s'il y a égalité.
pub fn swipe(_props: &str) -> Node {
    let n = IDEAS.len();
    let votes = use_state(vec![None::<bool>; n]);
    let index = use_state(0usize);
    let dx = use_state(0.0f64);
    let dy = use_state(0.0f64);
    let dragging = use_state(false);
    let start = use_state((0.0f64, 0.0f64));
    // Le tirage : l'idée éclairée pendant la roulette, puis le gagnant.
    let spotlight = use_state(None::<usize>);
    let drawn = use_state(None::<usize>);
    let seed = use_state(0x5EED_u64);

    let vote = move |yes: bool| {
        let i = index.get();
        if i >= n {
            return;
        }
        votes.update(|v| v[i] = Some(yes));
        dx.set(0.0);
        dy.set(0.0);
        index.set(i + 1);
        #[cfg(target_arch = "wasm32")]
        crate::client::haptic();
    };

    let reset = move || {
        votes.set(vec![None; n]);
        index.set(0);
        spotlight.set(None);
        drawn.set(None);
        seed.update(|s| *s = crate::content::mix(*s));
    };

    let cards = IDEAS.iter().enumerate().map(move |(i, idea)| {
        let likes = FRIENDS.iter().filter(|(_, v)| v[i]).count();
        div()
            .class("card")
            .class_if("is-top", move || index.get() == i)
            .class_if("is-dragging", move || dragging.get() && index.get() == i)
            .class_if("is-gone", move || index.get() > i)
            .style(format!("--hue:{}", idea.hue))
            .style_dyn("transform", move || {
                card_transform(
                    i as isize - index.get() as isize,
                    votes.with(|v| v[i]),
                    dx.get(),
                    dy.get(),
                )
            })
            .style_dyn("z-index", move || (n - i).to_string())
            .attr_dyn("aria-hidden", move || (index.get() != i).to_string())
            .on("pointerdown", move |e| {
                if index.get() != i {
                    return;
                }
                if let Some((x, y)) = pointer_xy(&e) {
                    start.set((x, y));
                    dragging.set(true);
                    capture(&e);
                }
            })
            .on("pointermove", move |e| {
                if !dragging.get() {
                    return;
                }
                if let Some((x, y)) = pointer_xy(&e) {
                    let (sx, sy) = start.get();
                    dx.set(x - sx);
                    dy.set(y - sy);
                }
            })
            .on("pointerup", move |_| {
                if !dragging.get() {
                    return;
                }
                dragging.set(false);
                let moved = dx.get();
                if moved.abs() > THRESHOLD {
                    vote(moved > 0.0);
                } else {
                    dx.set(0.0);
                    dy.set(0.0);
                }
            })
            .on("pointercancel", move |_| {
                dragging.set(false);
                dx.set(0.0);
                dy.set(0.0);
            })
            .child(
                span()
                    .class("card__stamp card__stamp--yes")
                    .attr("aria-hidden", "true")
                    .style_dyn("opacity", move || {
                        if index.get() == i {
                            format!("{:.2}", (dx.get() / THRESHOLD).clamp(0.0, 1.0))
                        } else {
                            "0".into()
                        }
                    })
                    .text("OUI"),
            )
            .child(
                span()
                    .class("card__stamp card__stamp--no")
                    .attr("aria-hidden", "true")
                    .style_dyn("opacity", move || {
                        if index.get() == i {
                            format!("{:.2}", (-dx.get() / THRESHOLD).clamp(0.0, 1.0))
                        } else {
                            "0".into()
                        }
                    })
                    .text("NON"),
            )
            .child(
                span()
                    .class("card__emoji")
                    .attr("aria-hidden", "true")
                    .text(idea.emoji),
            )
            .child(span().class("card__kind").text(idea.kind))
            .child(h3().class("card__title").text(idea.title))
            .child(p().class("card__detail").text(idea.detail))
            .child(
                p().class("card__by")
                    .text(format!("Proposé par {}", idea.by))
                    .child(span().class("card__likes").text(format!(
                        "{likes} ami{} pour",
                        if likes > 1 { "s" } else { "" }
                    ))),
            )
    });

    let results = move || {
        let mine = votes.get();
        let counts = tally(&mine);
        let max = counts.iter().copied().max().unwrap_or(1).max(1);
        let top = leaders(&counts);
        let tie = top.len() > 1;
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| counts[b].cmp(&counts[a]).then(a.cmp(&b)));
        let rows = order.into_iter().enumerate().map(move |(rank, i)| {
            let idea = IDEAS[i];
            li().class("rank")
                .class_if("is-leader", move || {
                    drawn.get() == Some(i) || (!tie && rank == 0)
                })
                .class_if("is-lit", move || spotlight.get() == Some(i))
                .style(format!(
                    "--hue:{};--w:{:.0}%;--d:{}ms",
                    idea.hue,
                    counts[i] as f64 / max as f64 * 100.0,
                    rank * 90
                ))
                .child(span().class("rank__emoji").text(idea.emoji))
                .child(span().class("rank__title").text(idea.title))
                .child(span().class("rank__bar").attr("aria-hidden", "true"))
                .child(
                    strong()
                        .class("rank__count")
                        .text(format!("{} oui", counts[i])),
                )
        });
        let verdict: Node = if !tie {
            let w = IDEAS[top[0]];
            p().class("verdict")
                .child(icon("crown"))
                .text(format!("Ce soir : {} {}", w.emoji, w.title))
                .into()
        } else if let Some(w) = drawn.get() {
            let w = IDEAS[w];
            p().class("verdict verdict--drawn")
                .child(icon("dice"))
                .text(format!("Tiré au sort : {} {}", w.emoji, w.title))
                .into()
        } else {
            let tied = top.clone();
            let picked = winner(&counts, seed.get()).unwrap_or(0);
            button()
                .class("btn btn--primary verdict__draw")
                .attr("type", "button")
                .on_click(move |_| {
                    #[cfg(target_arch = "wasm32")]
                    crate::client::roulette(spotlight, drawn, tied.clone(), picked);
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let _ = &tied;
                        drawn.set(Some(picked));
                    }
                })
                .child(icon("shuffle"))
                .text(format!("Égalité à {} : tirer au sort", top.len()))
                .into()
        };
        vec![
            div()
                .class("results")
                .child(h3().text("Résultat de la soirée"))
                .child(ol().class("ranking").children(rows))
                .child(verdict)
                .child(
                    button()
                        .class("btn btn--ghost")
                        .attr("type", "button")
                        .on_click(move |_| reset())
                        .child(icon("replay"))
                        .text("Revoter"),
                )
                .into(),
        ]
    };

    div()
        .class("demo")
        .child(
            div()
                .class("demo__people")
                .attr("aria-label", "Participants")
                .children(FRIENDS.iter().map(|(name, _)| {
                    span()
                        .class("avatar is-done")
                        .attr("title", format!("{name} a voté"))
                        .text(name.chars().next().unwrap_or('?').to_string())
                }))
                .child(
                    span()
                        .class("avatar avatar--me")
                        .class_if("is-done", move || index.get() >= n)
                        .attr("title", "Vous")
                        .text("Moi"),
                )
                .child(span().class("demo__progress").text_dyn(move || {
                    let i = index.get().min(n);
                    if i >= n {
                        "Tout le monde a voté ✓".to_owned()
                    } else {
                        format!("Carte {} sur {n}", i + 1)
                    }
                })),
        )
        .child(
            div()
                .class("deck")
                .class_if("is-done", move || index.get() >= n)
                .attr("tabindex", "0")
                .attr(
                    "aria-label",
                    "Pile d'idées : flèche droite pour oui, flèche gauche pour non",
                )
                .on("keydown", move |e| match e.key().as_str() {
                    "ArrowRight" => {
                        e.prevent_default();
                        vote(true);
                    }
                    "ArrowLeft" => {
                        e.prevent_default();
                        vote(false);
                    }
                    _ => {}
                })
                .children(cards)
                .child(div().class("deck__results").children_dyn(move || {
                    if index.get() >= n {
                        results()
                    } else {
                        Vec::new()
                    }
                })),
        )
        .child(
            div()
                .class("demo__actions")
                .class_if("is-hidden", move || index.get() >= n)
                .child(
                    button()
                        .class("round round--no")
                        .attr("type", "button")
                        .attr("aria-label", "Non")
                        .on_click(move |_| vote(false))
                        .child(icon("x")),
                )
                .child(
                    span()
                        .class("demo__hint")
                        .text("Glissez la carte ou touchez"),
                )
                .child(
                    button()
                        .class("round round--yes")
                        .attr("type", "button")
                        .attr("aria-label", "Oui")
                        .on_click(move |_| vote(true))
                        .child(icon("heart")),
                ),
        )
        .into()
}

/// La position du pointeur d'un événement.
#[cfg(target_arch = "wasm32")]
fn pointer_xy(e: &Event) -> Option<(f64, f64)> {
    use wasm_bindgen::JsCast;
    let m = e.raw().dyn_ref::<web_sys::MouseEvent>()?;
    Some((f64::from(m.client_x()), f64::from(m.client_y())))
}

#[cfg(not(target_arch = "wasm32"))]
fn pointer_xy(_e: &Event) -> Option<(f64, f64)> {
    None
}

/// La carte garde le pointeur, même s'il sort d'elle.
#[cfg(target_arch = "wasm32")]
fn capture(e: &Event) {
    use wasm_bindgen::JsCast;
    if let (Some(p), Some(target)) = (
        e.raw().dyn_ref::<web_sys::PointerEvent>(),
        e.current_target(),
    ) {
        let _ = target.set_pointer_capture(p.pointer_id());
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn capture(_e: &Event) {}

// ----- Le code de session ------------------------------------------------------------------

/// Les codes qui défilent.
pub const CODES: [&str; 4] = ["SOIREE", "K7Q2XM", "CINE42", "PIZZA9"];

/// Un code de session à six lettres, en panneau à palettes : les lettres basculent
/// à chaque nouvelle session.
pub fn flap(_props: &str) -> Node {
    let code = use_state(0usize);
    let tiles = div()
        .class("flap")
        .attr("role", "img")
        .attr_dyn("aria-label", move || {
            format!("Code de session : {}", CODES[code.get() % CODES.len()])
        })
        .on_mount(move |_el| {
            #[cfg(target_arch = "wasm32")]
            crate::client::every(2600, move || {
                if !code.is_alive() {
                    return false;
                }
                code.update(|c| *c = (*c + 1) % CODES.len());
                true
            });
        })
        .children_dyn(move || {
            CODES[code.get() % CODES.len()]
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    span()
                        .class("flap__tile")
                        .style(format!("--i:{}", i))
                        .child(span().class("flap__char").text(c.to_string()))
                        .into()
                })
                .collect()
        });
    div()
        .class("flap-wrap")
        .child(tiles)
        .child(
            p().class("ticket__url")
                .text_dyn(move || format!("Lien : …/s/{}", CODES[code.get() % CODES.len()])),
        )
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn islands_render_on_the_server() {
        for (name, island_fn) in ISLANDS {
            let html = paquets::active::island(name, "", island_fn).render();
            assert!(
                html.starts_with(&format!(r#"<active-island data-island="{name}""#)),
                "{name}: {html}"
            );
        }
        let html = paquets::active::island("swipe", "", swipe).render();
        assert_eq!(html.matches("class=\"card__emoji\"").count(), IDEAS.len());
        assert!(html.contains("Ramen chez Kodawari"));
        assert!(html.contains("Carte 1 sur 6"));
        let html = paquets::active::island("flap", "", flap).render();
        assert_eq!(html.matches("flap__tile").count(), 6);
        assert!(html.contains("Code de session : SOIREE"));
    }

    #[test]
    fn cards_fly_to_the_side_of_their_vote() {
        assert!(card_transform(-1, Some(true), 0.0, 0.0).starts_with("translate(160%"));
        assert!(card_transform(-1, Some(false), 0.0, 0.0).starts_with("translate(-160%"));
        assert!(card_transform(0, None, 42.0, 0.0).starts_with("translate(42.0px"));
        assert!(card_transform(2, None, 0.0, 0.0).contains("scale(0.900)"));
    }
}
