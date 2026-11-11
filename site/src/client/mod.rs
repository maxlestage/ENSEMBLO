//! Le bundle du navigateur : il réveille les îlots et lance les animations de la page.

pub mod canvas;
pub mod motion;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use paquets::active::State;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    paquets::hydrate(&crate::islands::ISLANDS);
    motion::init();
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("window")
}

/// Le visiteur a demandé moins d'animations.
pub fn reduced_motion() -> bool {
    matches("(prefers-reduced-motion: reduce)")
}

pub fn matches(query: &str) -> bool {
    window()
        .match_media(query)
        .ok()
        .flatten()
        .is_some_and(|m| m.matches())
}

/// Un écouteur pour toute la vie de la page, `passive` pour le défilement et le pointeur.
pub fn on<E: JsCast + 'static>(
    target: &web_sys::EventTarget,
    event: &str,
    passive: bool,
    f: impl FnMut(E) + 'static,
) {
    let mut f = f;
    let closure =
        Closure::<dyn FnMut(web_sys::Event)>::new(move |e: web_sys::Event| f(e.unchecked_into()));
    let options = web_sys::AddEventListenerOptions::new();
    options.set_passive(passive);
    let _ = target.add_event_listener_with_callback_and_add_event_listener_options(
        event,
        closure.as_ref().unchecked_ref(),
        &options,
    );
    closure.forget();
}

type FrameCallback = Closure<dyn FnMut(f64)>;

/// Appelle `f(instant)` à chaque image tant qu'elle rend `true`.
pub fn request_frame(f: impl FnMut(f64) -> bool + 'static) {
    let f = Rc::new(RefCell::new(f));
    let slot: Rc<RefCell<Option<FrameCallback>>> = Rc::new(RefCell::new(None));
    let again = slot.clone();
    *slot.borrow_mut() = Some(Closure::new(move |now: f64| {
        if (f.borrow_mut())(now) {
            if let Some(cb) = again.borrow().as_ref() {
                let _ = window().request_animation_frame(cb.as_ref().unchecked_ref());
            }
        } else {
            again.borrow_mut().take();
        }
    }));
    if let Some(cb) = slot.borrow().as_ref() {
        let _ = window().request_animation_frame(cb.as_ref().unchecked_ref());
    }
}

/// Appelle `f` toutes les `ms` millisecondes tant qu'elle rend `true`.
pub fn every(ms: i32, f: impl FnMut() -> bool + 'static) {
    let f = Rc::new(RefCell::new(f));
    let id = Rc::new(Cell::new(0));
    let stop = id.clone();
    let closure = Closure::<dyn FnMut()>::new(move || {
        if !(f.borrow_mut())() {
            window().clear_interval_with_handle(stop.get());
        }
    });
    if let Ok(handle) = window().set_interval_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        ms,
    ) {
        id.set(handle);
    }
    closure.forget();
}

/// Tous les éléments qui répondent à `selector`.
pub fn all(selector: &str) -> Vec<web_sys::HtmlElement> {
    let Ok(list) = window().document().unwrap().query_selector_all(selector) else {
        return Vec::new();
    };
    (0..list.length())
        .filter_map(|i| list.item(i)?.dyn_into().ok())
        .collect()
}

/// Une petite vibration sous le doigt, là où le navigateur le permet.
pub fn haptic() {
    let navigator = window().navigator();
    let vibrate = js_sys::Reflect::get(&navigator, &JsValue::from_str("vibrate"));
    if let Ok(f) = vibrate
        && let Some(f) = f.dyn_ref::<js_sys::Function>()
    {
        let _ = f.call1(&navigator, &JsValue::from_f64(12.0));
    }
}

/// La roulette du tirage au sort : la lumière passe d'un ex æquo à l'autre, de plus en
/// plus lentement, et s'arrête sur `picked`.
pub fn roulette(
    spotlight: State<Option<usize>>,
    drawn: State<Option<usize>>,
    tied: Vec<usize>,
    picked: usize,
) {
    if tied.is_empty() {
        return;
    }
    let landing = tied.iter().position(|&i| i == picked).unwrap_or(0);
    // Au moins trois tours, et l'arrêt sur le gagnant.
    let hops = tied.len() * 3 + landing + 1;
    if reduced_motion() {
        drawn.set(Some(picked));
        return;
    }
    let started = Cell::new(None::<f64>);
    request_frame(move |now| {
        if !spotlight.is_alive() {
            return false;
        }
        let t0 = *started.get().get_or_insert(now);
        started.set(Some(t0));
        // Chaque saut dure un peu plus que le précédent : la roulette ralentit.
        let elapsed = now - t0;
        let mut at = 0.0;
        let mut hop = 0;
        while hop < hops {
            let d = 60.0 + 260.0 * (hop as f64 / hops as f64).powi(2);
            if at + d > elapsed {
                break;
            }
            at += d;
            hop += 1;
        }
        if hop >= hops {
            spotlight.set(None);
            drawn.set(Some(picked));
            haptic();
            return false;
        }
        let lit = Some(tied[hop % tied.len()]);
        if spotlight.get() != lit {
            spotlight.set(lit);
        }
        true
    });
}
