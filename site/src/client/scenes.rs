//! Les scènes 3D ne tournent que lorsqu'elles sont à l'écran : hors champ, le jeu s'arrête
//! (et la batterie s'en porte mieux) ; quand il revient, il recommence du début.

use std::cell::RefCell;
use std::rc::Rc;

use paquets::ecs::Running;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit};

use crate::islands::Scene;

/// Ce qui arrête la scène et l'observation quand l'îlot s'en va.
pub struct Visible {
    observer: Option<IntersectionObserver>,
    running: Rc<RefCell<Option<Running>>>,
    _callback: Option<Closure<dyn Fn(js_sys::Array)>>,
}

impl Drop for Visible {
    fn drop(&mut self) {
        if let Some(observer) = &self.observer {
            observer.disconnect();
        }
        self.running.borrow_mut().take();
    }
}

fn run(canvas: &web_sys::Element, scene: Scene) -> Option<Running> {
    let still = super::reduced_motion();
    let app = match scene {
        Scene::Hero => crate::scenes::hero::app(still),
        Scene::Ballot(counter) => {
            counter.set(0);
            crate::scenes::ballot::app(Some(counter))
        }
    };
    match app.run_in(canvas) {
        Ok(running) => {
            let _ = canvas.class_list().add_1("is-live");
            Some(running)
        }
        Err(error) => {
            // Pas de WebGL 2 : le décor CSS reste à sa place.
            web_sys::console::warn_1(&error.into());
            let _ = canvas.class_list().add_1("is-off");
            None
        }
    }
}

pub fn start_when_visible(canvas: &web_sys::Element, scene: Scene) -> Visible {
    let running = Rc::new(RefCell::new(None::<Running>));
    let (slot, target) = (running.clone(), canvas.clone());
    let callback = Closure::<dyn Fn(js_sys::Array)>::new(move |entries: js_sys::Array| {
        for entry in entries.iter() {
            let Ok(entry) = entry.dyn_into::<IntersectionObserverEntry>() else {
                continue;
            };
            let mut slot = slot.borrow_mut();
            if entry.is_intersecting() {
                if slot.is_none() {
                    *slot = run(&target, scene);
                }
            } else if slot.take().is_some() {
                let _ = target.class_list().remove_1("is-live");
            }
        }
    });
    let options = IntersectionObserverInit::new();
    options.set_root_margin("80px 0px");
    let observer =
        IntersectionObserver::new_with_options(callback.as_ref().unchecked_ref(), &options).ok();
    match &observer {
        Some(observer) => observer.observe(canvas),
        // Sans IntersectionObserver, la scène tourne tout le temps.
        None => *running.borrow_mut() = run(canvas, scene),
    }
    Visible {
        observer,
        running,
        _callback: Some(callback),
    }
}
