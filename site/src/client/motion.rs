//! Les animations qui demandent le pointeur ou des nombres. Le reste (apparitions au
//! défilement, bandeau, panneau à palettes, parallaxe) est en CSS pur.

use std::cell::Cell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    HtmlElement, IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit,
    MouseEvent,
};

use super::{all, matches, on, reduced_motion, request_frame, window};

pub fn init() {
    let root: HtmlElement = window()
        .document()
        .and_then(|d| d.document_element())
        .expect("html")
        .unchecked_into();
    let _ = root.class_list().add_1("hydrated");
    reveals();
    counters();
    scroll_state(root);
    if matches("(pointer: fine)") && !reduced_motion() {
        tilt();
        glow();
    }
}

fn observe(
    selector: &str,
    threshold: f64,
    f: impl Fn(&IntersectionObserverEntry, &IntersectionObserver) + 'static,
) {
    let callback = Closure::<dyn Fn(js_sys::Array, IntersectionObserver)>::new(
        move |entries: js_sys::Array, io: IntersectionObserver| {
            for entry in entries.iter() {
                if let Ok(entry) = entry.dyn_into::<IntersectionObserverEntry>() {
                    f(&entry, &io);
                }
            }
        },
    );
    let options = IntersectionObserverInit::new();
    options.set_threshold(&JsValue::from_f64(threshold));
    options.set_root_margin("0px 0px -8% 0px");
    if let Ok(io) =
        IntersectionObserver::new_with_options(callback.as_ref().unchecked_ref(), &options)
    {
        for el in all(selector) {
            io.observe(&el);
        }
    }
    callback.forget();
}

/// Les éléments `data-reveal` apparaissent quand ils entrent à l'écran.
fn reveals() {
    observe("[data-reveal]", 0.12, |entry, io| {
        if entry.is_intersecting() {
            let _ = entry.target().class_list().add_1("is-in");
            io.unobserve(&entry.target());
        }
    });
}

/// Les nombres `data-count` comptent jusqu'à leur valeur quand ils apparaissent.
fn counters() {
    observe("[data-count]", 0.5, |entry, io| {
        if !entry.is_intersecting() {
            return;
        }
        let el: HtmlElement = entry.target().unchecked_into();
        io.unobserve(&el);
        let target: f64 = el
            .get_attribute("data-count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);
        let suffix = el.get_attribute("data-suffix").unwrap_or_default();
        if reduced_motion() {
            el.set_text_content(Some(&format!("{target}{suffix}")));
            return;
        }
        let start = Cell::new(None::<f64>);
        request_frame(move |now| {
            let t0 = *start.get().get_or_insert(now);
            start.set(Some(t0));
            let p = ((now - t0) / 1400.0).min(1.0);
            let eased = 1.0 - (1.0 - p).powi(4);
            let value = (target * eased).round();
            el.set_text_content(Some(&format!("{value}{suffix}")));
            p < 1.0
        });
    });
}

/// L'en-tête change au défilement, et `--scroll` (0 à 1) suit la page.
fn scroll_state(root: HtmlElement) {
    let ticking = Rc::new(Cell::new(false));
    let update = Rc::new(move || {
        let w = window();
        let y = w.scroll_y().unwrap_or(0.0);
        let height = w
            .inner_height()
            .ok()
            .and_then(|h| h.as_f64())
            .unwrap_or(800.0);
        let _ = root.class_list().toggle_with_force("is-scrolled", y > 24.0);
        let max = f64::from(root.scroll_height()) - height;
        let p = if max > 0.0 { y / max } else { 0.0 };
        let _ = root.style().set_property("--scroll", &format!("{p:.4}"));
    });
    update();
    on(&window(), "scroll", true, move |_: web_sys::Event| {
        if ticking.replace(true) {
            return;
        }
        let (update, ticking) = (update.clone(), ticking.clone());
        request_frame(move |_| {
            ticking.set(false);
            update();
            false
        });
    });
}

/// Les éléments `data-tilt` penchent vers la souris, en 3D.
fn tilt() {
    for el in all("[data-tilt]") {
        let target = el.clone();
        on(&el, "pointermove", true, move |e: MouseEvent| {
            let rect = target.get_bounding_client_rect();
            let x = (f64::from(e.client_x()) - rect.left()) / rect.width() - 0.5;
            let y = (f64::from(e.client_y()) - rect.top()) / rect.height() - 0.5;
            let style = target.style();
            let _ = style.set_property("--ry", &format!("{:.2}deg", x * 14.0));
            let _ = style.set_property("--rx", &format!("{:.2}deg", -y * 14.0));
        });
        let target = el.clone();
        on(&el, "pointerleave", true, move |_: MouseEvent| {
            let style = target.style();
            let _ = style.set_property("--ry", "0deg");
            let _ = style.set_property("--rx", "0deg");
        });
    }
}

/// Un halo suit la souris sur les éléments `data-glow`.
fn glow() {
    for el in all("[data-glow]") {
        let target = el.clone();
        on(&el, "pointermove", true, move |e: MouseEvent| {
            let rect = target.get_bounding_client_rect();
            let style = target.style();
            let _ = style.set_property(
                "--mx",
                &format!("{:.0}px", f64::from(e.client_x()) - rect.left()),
            );
            let _ = style.set_property(
                "--my",
                &format!("{:.0}px", f64::from(e.client_y()) - rect.top()),
            );
        });
    }
}
