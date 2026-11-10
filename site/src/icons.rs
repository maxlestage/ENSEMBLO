//! Les icônes du site : du SVG en ligne, au trait, qui prend la couleur du texte.

use paquets::prelude::*;

/// Le tracé d'une icône (grille de 24 × 24).
fn path(name: &str) -> &'static str {
    match name {
        "spark" => {
            r#"<path d="M12 3v4M12 17v4M3 12h4M17 12h4M5.6 5.6l2.8 2.8M15.6 15.6l2.8 2.8M18.4 5.6l-2.8 2.8M8.4 15.6l-2.8 2.8"/>"#
        }
        "qr" => {
            r#"<rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><path d="M14 14h3v3h-3zM20 14v.01M14 20h.01M17 17h4v4h-4"/>"#
        }
        "bulb" => {
            r#"<path d="M9 18h6M10 21h4M12 3a6 6 0 0 0-3.6 10.8c.6.5 1.1 1.3 1.1 2.2h5c0-.9.5-1.7 1.1-2.2A6 6 0 0 0 12 3z"/>"#
        }
        "swipe" => {
            r#"<rect x="6" y="3" width="12" height="16" rx="3" transform="rotate(-8 12 11)"/><path d="M3 21h4M17 21h4M5 19l-2 2 2 2M19 19l2 2-2 2"/>"#
        }
        "crown" => r#"<path d="M3 8l4.5 4L12 5l4.5 7L21 8l-2 11H5z"/><path d="M5 19h14"/>"#,
        "bolt" => r#"<path d="M13 2 4 14h7l-1 8 9-12h-7z"/>"#,
        "lock" => {
            r#"<rect x="4" y="10" width="16" height="11" rx="2.5"/><path d="M8 10V7a4 4 0 0 1 8 0v3M12 15v2"/>"#
        }
        "phone" => {
            r#"<rect x="6" y="2" width="12" height="20" rx="3"/><path d="M10 5h4M11 18.5h2"/>"#
        }
        "wifi" => {
            r#"<path d="M2 9a15 15 0 0 1 20 0M5 12.5a10 10 0 0 1 14 0M8.5 16a5 5 0 0 1 7 0"/><circle cx="12" cy="19.5" r="1"/>"#
        }
        "dice" => {
            r#"<rect x="3" y="3" width="18" height="18" rx="4"/><circle cx="8" cy="8" r="1.2"/><circle cx="16" cy="16" r="1.2"/><circle cx="16" cy="8" r="1.2"/><circle cx="8" cy="16" r="1.2"/><circle cx="12" cy="12" r="1.2"/>"#
        }
        "leaf" => r#"<path d="M5 19c0-9 6-15 15-15 0 9-6 15-15 15z"/><path d="M5 19 13 11"/>"#,
        "apple" => {
            r#"<path d="M16.5 12.6c0-2.4 2-3.5 2.1-3.6-1.1-1.7-2.9-1.9-3.5-1.9-1.5-.2-2.9.9-3.7.9s-1.9-.9-3.2-.8c-1.6 0-3.1 1-4 2.4-1.7 3-.4 7.4 1.2 9.8.8 1.2 1.8 2.5 3 2.4 1.2 0 1.7-.8 3.1-.8s1.9.8 3.2.8c1.3 0 2.1-1.2 2.9-2.4.9-1.3 1.3-2.6 1.3-2.7 0 0-2.4-.9-2.4-4.1zM14.1 5.4c.7-.8 1.1-1.9 1-3-1 0-2.1.7-2.8 1.5-.6.7-1.2 1.8-1 2.9 1 .1 2.1-.6 2.8-1.4z" fill="currentColor" stroke="none"/>"#
        }
        "arrow" => r#"<path d="M5 12h14M13 6l6 6-6 6"/>"#,
        "check" => r#"<path d="M5 12.5 10 17.5 19.5 7"/>"#,
        "x" => r#"<path d="M6 6l12 12M18 6 6 18"/>"#,
        "heart" => {
            r#"<path d="M12 20s-7.5-4.6-9.3-9.2C1.4 7.5 3.5 4 7 4c2 0 3.6 1.2 5 3 1.4-1.8 3-3 5-3 3.5 0 5.6 3.5 4.3 6.8C19.5 15.4 12 20 12 20z"/>"#
        }
        "shuffle" => {
            r#"<path d="M3 7h3c4 0 6 10 10 10h5M3 17h3c1.7 0 3-1.7 4.2-3.8M14 10c1-1.8 2.3-3 4-3h3M18 4l3 3-3 3M18 14l3 3-3 3"/>"#
        }
        "replay" => r#"<path d="M4 12a8 8 0 1 0 2.4-5.7L4 8.7M4 4v4.7h4.7"/>"#,
        "users" => {
            r#"<circle cx="9" cy="8" r="3.5"/><path d="M2.5 20c.6-3.6 3.2-5.5 6.5-5.5s5.9 1.9 6.5 5.5"/><path d="M16 4.6a3.5 3.5 0 0 1 0 6.8M18 14.8c2 .7 3.2 2.4 3.5 5.2"/>"#
        }
        "menu" => r#"<path d="M4 7h16M4 12h16M4 17h10"/>"#,
        _ => r#"<circle cx="12" cy="12" r="8"/>"#,
    }
}

/// Une icône décorative (le texte à côté dit ce qu'elle veut dire).
pub fn icon(name: &str) -> Element {
    span().class("icon").attr("aria-hidden", "true").html(format!(
        r#"<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">{}</svg>"#,
        path(name)
    ))
}

/// Le logo d'Ensemblo : un cube dont les faces sont des cartes.
pub fn logo() -> Element {
    span().class("logo").attr("aria-hidden", "true").html(
        r##"<svg viewBox="0 0 64 64" width="32" height="32"><defs><linearGradient id="lg" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#ff6b6b"/><stop offset=".5" stop-color="#a66bff"/><stop offset="1" stop-color="#3ee0d0"/></linearGradient></defs><path fill="url(#lg)" d="M32 4 58 17v30L32 60 6 47V17z"/><path fill="#fff" fill-opacity=".85" d="M32 4 58 17 32 30 6 17z"/><path fill="#000" fill-opacity=".2" d="M32 30v30l26-13V17z"/><path d="M24 15.5 32 11l8 4.5-8 4.5z" fill="#ff6b6b" fill-opacity=".9"/></svg>"##,
    )
}
