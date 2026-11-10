//! Le serveur : les pages rendues par Paquets (HTML fini, balises SEO, CSP), la PWA, le
//! plan du site et les fichiers de `public/` et `pkg/`.

use std::path::{Path, PathBuf};

use paquets::prelude::*;

use crate::content::{FAQ, FEATURES, IDEAS, MARQUEE, ROTATING, STEPS};
use crate::icons::{icon, logo};
use crate::islands::{ballot3d, flap, hero3d, swipe};

pub const NAME: &str = "Ensemblo";
pub const TAGLINE: &str = "On fait quoi ce soir ?";
pub const DESCRIPTION: &str = "Ensemblo met fin aux débats sans fin : créez une session, partagez le QR‑code, chacun propose ses idées et vote en swipant. L'idée gagnante s'affiche en direct, un tirage au sort départage les égalités.";
const FONTS: &str = "https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wght@12..96,400..800&family=Inter:wght@400..700&display=swap";

/// Le dossier qui contient `public/` (et `pkg/`).
pub fn site_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("SITE_DIR") {
        return PathBuf::from(dir);
    }
    if Path::new("public").is_dir() {
        return PathBuf::from(".");
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Une empreinte courte du contenu, pour que les navigateurs rechargent la feuille de style
/// quand elle change.
pub fn version(bytes: &[u8]) -> String {
    let h = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3)
    });
    format!("{h:016x}")[..10].to_owned()
}

fn css_href() -> String {
    static HREF: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    HREF.get_or_init(|| {
        let css = std::fs::read(site_dir().join("public/site.css")).unwrap_or_default();
        format!("/site.css?v={}", version(&css))
    })
    .clone()
}

/// L'adresse publique du site : `SITE_URL`, ou celle de la requête.
fn origin(req: &Request) -> String {
    if let Ok(url) = std::env::var("SITE_URL") {
        let url = url.trim().trim_end_matches('/');
        if !url.is_empty() {
            return url.to_owned();
        }
    }
    let header = |name: &str| {
        req.header(name)
            .map(|v| v.split(',').next().unwrap_or("").trim().to_owned())
    };
    let host = header("x-forwarded-host")
        .or_else(|| header("host"))
        .filter(|h| {
            !h.is_empty()
                && h.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':'))
        })
        .unwrap_or_else(|| "localhost".into());
    let proto = if header("x-forwarded-proto").as_deref() == Some("https") {
        "https"
    } else {
        "http"
    };
    format!("{proto}://{host}")
}

/// Une page du site : thème, polices, en-tête et pied de page.
fn page(title: &str, description: &str, origin: &str, path: &str) -> Page {
    let full = if title == NAME {
        format!("{NAME} · {TAGLINE}")
    } else {
        format!("{title} · {NAME}")
    };
    Page::new(full)
        .lang("fr")
        .plain()
        .description(description)
        .seo(
            Seo::new()
                .canonical(format!("{origin}{path}"))
                .image(format!("{origin}/og-image.png"))
                .site_name(NAME)
                .locale("fr_FR")
                .og_type("website")
                .json_ld(
                    json!({
                        "@context": "https://schema.org",
                        "@type": "MobileApplication",
                        "name": NAME,
                        "description": DESCRIPTION,
                        "applicationCategory": "LifestyleApplication",
                        "operatingSystem": "iOS, Web",
                        "inLanguage": "fr",
                        "offers": { "@type": "Offer", "price": "0", "priceCurrency": "EUR" }
                    })
                    .to_string(),
                ),
        )
        .head(r##"<link rel="icon" href="/favicon.svg" type="image/svg+xml"/><meta name="color-scheme" content="dark"/><meta name="theme-color" content="#0d0a1f"/>"##)
        .stylesheet(FONTS)
        .stylesheet(css_href())
        .view_transitions()
        .body(header_bar())
}

fn header_bar() -> Element {
    header()
        .class("topbar")
        .child(
            a().class("brand")
                .href("/")
                .attr("aria-label", "Ensemblo, accueil")
                .child(logo())
                .child(span().text(NAME)),
        )
        .child(
            nav()
                .class("topbar__nav")
                .attr("aria-label", "Sections")
                .child(a().href("/#concept").text("Concept"))
                .child(a().href("/#demo").text("Démo"))
                .child(a().href("/#ios").text("iPhone"))
                .child(a().href("/#faq").text("FAQ")),
        )
        .child(
            a().class("btn btn--small btn--primary topbar__cta")
                .href("/#demo")
                .text("Essayer"),
        )
}

fn footer_bar() -> Element {
    footer()
        .class("footer")
        .child(
            div()
                .class("footer__brand")
                .child(logo())
                .child(strong().text(NAME))
                .child(span().text(TAGLINE)),
        )
        .child(
            nav()
                .class("footer__links")
                .attr("aria-label", "Pied de page")
                .child(a().href("/#concept").text("Concept"))
                .child(a().href("/#demo").text("Démo"))
                .child(a().href("/#ios").text("App iPhone"))
                .child(a().href("/mentions-legales").text("Mentions légales")),
        )
        .child(
            p().class("footer__small")
                .text("Site écrit en Rust avec Paquets · App iOS en Swift · © 2026 Ensemblo"),
        )
}

/// Le titre d'une section, avec son surtitre.
fn heading(eyebrow: &'static str, title: &'static str, lead: &'static str) -> Element {
    div()
        .class("heading")
        .attr("data-reveal", "")
        .child(p().class("eyebrow").text(eyebrow))
        .child(h2().text(title))
        .child(p().class("lead").text(lead))
}

// ----- L'accueil ----------------------------------------------------------------------------

fn hero() -> Element {
    let rotator = span().class("rotator").attr("aria-hidden", "true").child(
        span()
            .class("rotator__track")
            .style(format!("--n:{}", ROTATING.len()))
            .children(
                ROTATING
                    .iter()
                    .chain(ROTATING.first())
                    .map(|w| span().class("rotator__word").text(*w)),
            ),
    );
    section()
        .class("hero")
        .attr("aria-labelledby", "hero-title")
        .child(
            div()
                .class("aurora")
                .attr("aria-hidden", "true")
                .child(span())
                .child(span())
                .child(span()),
        )
        .child(
            div()
                .class("hero__copy")
                .child(
                    p().class("pill")
                        .child(span().class("pill__dot"))
                        .text("Décision de groupe, en temps réel"),
                )
                .child(
                    h1().id("hero-title")
                        .child(span().class("hero__line").text("On fait quoi"))
                        .child(span().class("hero__line hero__line--grad").text("ce soir ?")),
                )
                .child(
                    p().class("hero__lead")
                        .text("Plutôt ")
                        .child(rotator)
                        .child(span().class("sr-only").text("un ciné, un resto, une balade"))
                        .text(" ? Chacun propose, tout le monde swipe, Ensemblo tranche. Fini les « je sais pas, et toi ? ».")
                )
                .child(
                    div()
                        .class("cta")
                        .child(
                            a().class("btn btn--primary btn--big")
                                .href("#demo")
                                .attr("data-glow", "")
                                .text("Essayer la démo")
                                .child(icon("arrow")),
                        )
                        .child(
                            a().class("btn btn--ghost btn--big")
                                .href("#ios")
                                .child(icon("apple"))
                                .text("L'app iPhone"),
                        ),
                )
                .child(
                    ul().class("stats")
                        .child(stat("5", "", "étapes, de l'idée au verdict"))
                        .child(stat("0", "", "compte à créer"))
                        .child(stat("24", " h", "avant que la session expire")),
                ),
        )
        .child(
            div()
                .class("hero__stage")
                .child(
                    div()
                        .class("orbit-cards")
                        .attr("aria-hidden", "true")
                        .children(IDEAS.iter().take(3).enumerate().map(|(i, idea)| {
                            div()
                                .class("float-card")
                                .style(format!("--i:{};--hue:{}", i, idea.hue))
                                .child(span().class("float-card__emoji").text(idea.emoji))
                                .child(span().class("float-card__title").text(idea.kind))
                        })),
                )
                .child(island("hero3d", "", hero3d))
                .child(
                    p().class("hero__hint")
                        .child(icon("swipe"))
                        .text("Touchez le plateau pour voter, glissez pour tourner"),
                ),
        )
}

fn stat(value: &'static str, suffix: &'static str, label: &'static str) -> Element {
    li().class("stat")
        .child(
            strong()
                .attr("data-count", value)
                .attr("data-suffix", suffix)
                .text(format!("{value}{suffix}")),
        )
        .child(span().text(label))
}

fn marquee() -> Element {
    let items = || {
        MARQUEE
            .iter()
            .map(|item| span().class("marquee__item").text(*item))
    };
    div().class("marquee").attr("aria-hidden", "true").child(
        div()
            .class("marquee__track")
            .children(items())
            .children(items()),
    )
}

fn concept() -> Element {
    section()
        .class("section")
        .id("concept")
        .child(heading(
            "Le concept",
            "Cinq étapes, zéro débat",
            "De « on fait quoi ? » à « c'est parti ! » en moins de deux minutes, sans compte et sans installation pour vos amis.",
        ))
        .child(
            ol().class("steps")
                .children(STEPS.iter().enumerate().map(|(i, (ico, title, text))| {
                    li().class("step")
                        .attr("data-reveal", "")
                        .attr("data-tilt", "")
                        .style(format!("--d:{}ms", i * 110))
                        .child(
                            div()
                                .class("step__inner")
                                .child(span().class("step__num").text(format!("0{}", i + 1)))
                                .child(span().class("step__icon").child(icon(ico)))
                                .child(h3().text(*title))
                                .child(p().text(*text)),
                        )
                })),
        )
}

/// Un QR‑code décoratif (il ne s'ouvre pas : c'est une illustration).
fn qr_art(seed: u64) -> String {
    const N: usize = 25;
    let finder = |x: usize, y: usize| {
        let near = |a: usize, b: usize| x >= a && x < a + 7 && y >= b && y < b + 7;
        [(0, 0), (N - 7, 0), (0, N - 7)]
            .iter()
            .find(|&&(a, b)| near(a, b))
            .map(|&(a, b)| {
                let (dx, dy) = (x - a, y - b);
                let ring = dx == 0 || dy == 0 || dx == 6 || dy == 6;
                let core = (2..=4).contains(&dx) && (2..=4).contains(&dy);
                ring || core
            })
    };
    let mut svg = format!(
        r##"<svg viewBox="-2 -2 {s} {s}" width="100%" height="100%" shape-rendering="crispEdges"><rect x="-2" y="-2" width="{s}" height="{s}" rx="2" fill="#fff"/>"##,
        s = N + 4
    );
    let mut state = seed;
    for y in 0..N {
        for x in 0..N {
            let on = finder(x, y).unwrap_or_else(|| {
                state = crate::content::mix(state);
                // Les séparateurs autour des repères restent blancs.
                let clear = (x < 8 && y < 8) || (x >= N - 8 && y < 8) || (x < 8 && y >= N - 8);
                !clear && state % 100 < 47
            });
            if on {
                svg.push_str(&format!(
                    r##"<rect x="{x}" y="{y}" width="1" height="1" fill="#120c2b"/>"##
                ));
            }
        }
    }
    svg.push_str("</svg>");
    svg
}

fn join() -> Element {
    section()
        .class("section section--split")
        .id("rejoindre")
        .child(
            div()
                .class("split__copy")
                .child(heading(
                    "Rejoindre",
                    "Un code, un QR, un lien",
                    "Chaque session a son code à six caractères. Les amis scannent le QR‑code avec l'appareil photo, ou tapent le code : ils sont dedans, depuis un iPhone, un Android ou un ordinateur.",
                ))
                .child(
                    ul().class("checks")
                        .attr("data-reveal", "")
                        .child(li().child(icon("check")).text("Un scan du QR‑code ouvre directement la session"))
                        .child(li().child(icon("check")).text("Partage en un geste : Messages, WhatsApp, AirDrop"))
                        .child(li().child(icon("check")).text("La session expire au bout de 24 heures")),
                ),
        )
        .child(
            div()
                .class("ticket-stage")
                .attr("data-reveal", "")
                .child(
                    div()
                        .class("ticket")
                        .attr("data-tilt", "")
                        .child(
                            div()
                                .class("ticket__face")
                                .child(p().class("ticket__label").text("Session de ce soir"))
                                .child(island("flap", "", flap))
                                .child(
                                    div()
                                        .class("ticket__qr")
                                        .attr("aria-hidden", "true")
                                        .html(qr_art(0x0E5E_7B10)),
                                )
                                .child(p().class("ticket__note").text("Illustration")),
                        ),
                ),
        )
}

fn demo() -> Element {
    section()
        .class("section section--demo")
        .id("demo")
        .child(heading(
            "Démo interactive",
            "Swipez la soirée",
            "Alice, Bastien et Chloé ont déjà voté. À vous : glissez chaque carte à droite pour oui, à gauche pour non (ou utilisez les flèches du clavier).",
        ))
        .child(
            div()
                .class("demo-stage")
                .attr("data-reveal", "")
                .child(island("swipe", "", swipe)),
        )
}

/// Un téléphone en CSS 3D, avec une mini-pile de cartes à l'écran.
fn mini_phone(i: usize, name: &str) -> Element {
    div()
        .class("phone")
        .style(format!("--i:{}", i))
        .attr("aria-hidden", "true")
        .child(
            div()
                .class("phone__screen")
                .child(div().class("phone__notch"))
                .child(p().class("phone__name").text(name.to_owned()))
                .child(div().class("phone__stack").children(
                    IDEAS.iter().skip(i).take(3).enumerate().map(|(k, idea)| {
                        div()
                            .class("mini-card")
                            .style(format!("--k:{};--hue:{}", k, idea.hue))
                            .child(span().text(idea.emoji))
                    }),
                ))
                .child(
                    div()
                        .class("phone__votes")
                        .child(span().class("vote-dot vote-dot--yes"))
                        .child(span().class("vote-dot vote-dot--no"))
                        .child(span().class("vote-dot vote-dot--yes")),
                ),
        )
}

fn realtime() -> Element {
    section()
        .class("section")
        .id("temps-reel")
        .child(heading(
            "Temps réel",
            "Tous les écrans, à la même seconde",
            "Une idée ajoutée apparaît chez tout le monde. Un vote tombe, la jauge bouge. Firebase Realtime Database fait circuler la session, sans serveur à gérer.",
        ))
        .child(
            div()
                .class("phones")
                .attr("data-reveal", "")
                .child(mini_phone(0, "Alice"))
                .child(mini_phone(1, "Vous"))
                .child(mini_phone(2, "Bastien"))
                .child(div().class("pulse").attr("aria-hidden", "true")),
        )
        .child(
            ul().class("features")
                .children(FEATURES.iter().enumerate().map(|(i, (ico, title, text))| {
                    li().class("feature")
                        .attr("data-reveal", "")
                        .attr("data-glow", "")
                        .style(format!("--d:{}ms", (i % 3) * 100))
                        .child(span().class("feature__icon").child(icon(ico)))
                        .child(h3().text(*title))
                        .child(p().text(*text))
                })),
        )
}

fn ballot() -> Element {
    section()
        .class("section")
        .id("resultat")
        .child(heading(
            "Le résultat",
            "Le verdict tombe, en 3D",
            "Chaque oui est une bille qui tombe dans l'urne de son idée. La plus remplie gagne. En cas d'égalité, un tirage au sort départage les ex æquo, avec la même graine sur chaque téléphone.",
        ))
        .child(
            div()
                .class("ballot-stage")
                .attr("data-reveal", "")
                .child(island("ballot3d", "", ballot3d)),
        )
}

/// L'iPhone en CSS 3D, avec l'écran de vote de l'app.
fn iphone() -> Element {
    let idea = IDEAS[1];
    div()
        .class("iphone-stage")
        .attr("data-reveal", "")
        .attr("aria-hidden", "true")
        .child(
            div()
                .class("iphone")
                .child(div().class("iphone__side iphone__side--right"))
                .child(div().class("iphone__side iphone__side--left"))
                .child(div().class("iphone__back"))
                .child(
                    div()
                        .class("iphone__front")
                        .child(div().class("iphone__island"))
                        .child(
                            div()
                                .class("app")
                                .child(
                                    div()
                                        .class("app__top")
                                        .child(span().class("app__code").text("SOIREE"))
                                        .child(span().class("app__people").text("4 ✓")),
                                )
                                .child(p().class("app__title").text("On fait quoi ce soir ?"))
                                .child(
                                    div()
                                        .class("app__card")
                                        .style(format!("--hue:{}", idea.hue))
                                        .child(span().class("app__emoji").text(idea.emoji))
                                        .child(strong().text(idea.title))
                                        .child(small().text(idea.detail)),
                                )
                                .child(
                                    div()
                                        .class("app__buttons")
                                        .child(
                                            span().class("app__btn app__btn--no").child(icon("x")),
                                        )
                                        .child(
                                            span()
                                                .class("app__btn app__btn--yes")
                                                .child(icon("heart")),
                                        ),
                                ),
                        ),
                ),
        )
}

fn ios() -> Element {
    section()
        .class("section section--split section--ios")
        .id("ios")
        .child(iphone())
        .child(
            div()
                .class("split__copy")
                .child(heading(
                    "L'app iPhone",
                    "Native, en Swift",
                    "SwiftUI, Swift 6 et ses vérifications de concurrence, le SDK Firebase pour Apple : une app légère qui s'ouvre en un instant et suit la session en direct.",
                ))
                .child(
                    ul().class("checks")
                        .attr("data-reveal", "")
                        .child(li().child(icon("check")).text("Swipe à la main, avec retour haptique à chaque vote"))
                        .child(li().child(icon("check")).text("QR‑code de la session, et scanner intégré pour rejoindre"))
                        .child(li().child(icon("check")).text("Partage du lien par la feuille de partage d'iOS"))
                        .child(li().child(icon("check")).text("Votes gardés hors ligne, envoyés au retour du réseau"))
                        .child(li().child(icon("check")).text("Tirage au sort animé, identique sur tous les appareils")),
                )
                .child(
                    p().class("store-badge")
                        .attr("data-reveal", "")
                        .child(icon("apple"))
                        .child(
                            span()
                                .child(small().text("Bientôt sur"))
                                .child(strong().text("l'App Store")),
                        ),
                ),
        )
}

fn tech() -> Element {
    const MODEL: &str = r#"sessions/K7Q2XM
├─ createdAt    1731276000000
├─ creator      { id, name: "Alice" }
├─ participants { userId: "Bastien", … }
├─ seed         2718281828
└─ items/itemId
   ├─ text       "Ramen chez Kodawari"
   ├─ createdBy  "userId2"
   └─ votes      { userId1: true, userId2: false }"#;
    section()
        .class("section")
        .id("technique")
        .child(heading(
            "Sous le capot",
            "Simple, rapide, sans serveur",
            "Une session est un petit arbre dans Firebase Realtime Database. Chaque appareil l'écoute et le met à jour : rien d'autre à héberger.",
        ))
        .child(
            div()
                .class("tech")
                .child(
                    div()
                        .class("window")
                        .attr("data-reveal", "")
                        .child(
                            div()
                                .class("window__bar")
                                .child(span())
                                .child(span())
                                .child(span())
                                .child(em().text("Firebase · Realtime Database")),
                        )
                        .child(pre().class("code").child(code().text(MODEL))),
                )
                .child(
                    ul().class("stack")
                        .attr("data-reveal", "")
                        .style("--d:120ms")
                        .child(chip("Rust + Paquets", "ce site, ses îlots WebAssembly et sa 3D"))
                        .child(chip("Swift 6 + SwiftUI", "l'app iPhone, iOS 26 et plus"))
                        .child(chip("Firebase", "Realtime Database, connexion anonyme"))
                        .child(chip("PWA", "installable, utilisable hors ligne")),
                ),
        )
}

fn chip(title: &str, text: &str) -> Element {
    li().class("chip")
        .child(strong().text(title.to_owned()))
        .child(span().text(text.to_owned()))
}

fn faq() -> Element {
    section()
        .class("section section--narrow")
        .id("faq")
        .child(heading(
            "Questions",
            "Tout ce qu'on nous demande",
            "Et si la réponse n'est pas là, créez une session pour en débattre.",
        ))
        .child(
            div()
                .class("faq")
                .children(FAQ.iter().enumerate().map(|(i, (q, a))| {
                    details()
                        .class("faq__item")
                        .attr("data-reveal", "")
                        .style(format!("--d:{}ms", i * 60))
                        .child(
                            summary()
                                .text(*q)
                                .child(span().class("faq__plus").attr("aria-hidden", "true")),
                        )
                        .child(p().text(*a))
                })),
        )
}

fn finale() -> Element {
    section().class("finale").child(
        div()
            .class("finale__inner")
            .attr("data-reveal", "")
            .child(h2().text("Ce soir, on décide ensemble."))
            .child(p().text("Créez une session, envoyez le QR‑code, et laissez les swipes parler."))
            .child(
                a().class("btn btn--light btn--big")
                    .href("#demo")
                    .text("Lancer la démo")
                    .child(icon("arrow")),
            ),
    )
}

async fn home(req: Request) -> Page {
    let origin = origin(&req);
    page(NAME, DESCRIPTION, &origin, "/")
        .body(
            html::main()
                .id("contenu")
                .child(hero())
                .child(marquee())
                .child(concept())
                .child(join())
                .child(demo())
                .child(realtime())
                .child(ballot())
                .child(ios())
                .child(tech())
                .child(faq())
                .child(finale()),
        )
        .body(footer_bar())
}

async fn legal(req: Request) -> Page {
    let origin = origin(&req);
    page(
        "Mentions légales",
        "Mentions légales et données personnelles d'Ensemblo.",
        &origin,
        "/mentions-legales",
    )
    .body(
        html::main().class("prose").child(
            article()
                .child(h1().text("Mentions légales"))
                .child(h2().text("Éditeur"))
                .child(p().text("Ensemblo, application de décision de groupe."))
                .child(h2().text("Données"))
                .child(p().text("Une session contient des prénoms, des idées et des votes. Elle est enregistrée dans Firebase Realtime Database avec un identifiant anonyme ; elle n'est plus lisible 24 heures après sa création, puis l'app de son créateur l'efface. Ce site ne dépose aucun cookie et n'utilise aucun outil de mesure d'audience."))
                .child(h2().text("Hébergement"))
                .child(p().text("Le site est un serveur Rust écrit avec Paquets, servi depuis un conteneur."))
                .child(p().child(a().class("link-arrow").href("/").text("Retour à l'accueil").child(icon("arrow")))),
        ),
    )
    .body(footer_bar())
}

async fn not_found(req: Request) -> Page {
    let origin = origin(&req);
    page(
        "Page introuvable",
        "Cette page n'existe pas.",
        &origin,
        "/404",
    )
    .status(404)
    .body(
        html::main().class("prose prose--center").child(
            article()
                .child(p().class("big-emoji").text("🤷"))
                .child(h1().text("On ne sait pas non plus."))
                .child(p().text("Cette page n'existe pas. Mais on peut voter pour une autre."))
                .child(
                    a().class("btn btn--primary")
                        .href("/")
                        .text("Retour à l'accueil")
                        .child(icon("arrow")),
                ),
        ),
    )
    .body(footer_bar())
}

async fn sitemap(req: Request) -> Response {
    let origin = origin(&req);
    let urls: String = ["/", "/mentions-legales"]
        .iter()
        .map(|p| format!("<url><loc>{origin}{p}</loc></url>\n"))
        .collect();
    Response::new()
        .header("content-type", "application/xml; charset=utf-8")
        .send(format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{urls}</urlset>\n"
        ))
}

async fn robots(req: Request) -> Response {
    let origin = origin(&req);
    Response::new()
        .header("content-type", "text/plain; charset=utf-8")
        .send(format!(
            "User-agent: *\nAllow: /\n\nSitemap: {origin}/sitemap.xml\n"
        ))
}

pub fn app() -> App {
    let mut app = App::new();
    app.get("/", home);
    app.get("/mentions-legales", legal);
    app.get("/sitemap.xml", sitemap);
    app.get("/robots.txt", robots);
    app.health("/health");
    app.pwa(
        Pwa::new(NAME)
            .short_name(NAME)
            .description(TAGLINE)
            .start_url("/")
            .theme_color("#0d0a1f")
            .background_color("#0d0a1f")
            .lang("fr")
            .categories(["lifestyle", "social", "entertainment"]),
    );
    app.fallback(not_found);
    app
}

/// Lance le serveur depuis le dossier du site (`PORT` sur Heroku, 3000 sinon).
pub fn run() -> std::io::Result<()> {
    let dir = site_dir();
    if dir != Path::new(".") {
        std::env::set_current_dir(&dir)?;
    }
    app().run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_qr_art_has_its_three_finders() {
        let svg = qr_art(1);
        assert!(svg.starts_with("<svg"));
        // Un repère : 24 modules d'anneau et 9 de cœur, trois fois.
        assert!(svg.matches("<rect").count() > 3 * 33);
        assert_eq!(qr_art(1), qr_art(1));
    }
}
