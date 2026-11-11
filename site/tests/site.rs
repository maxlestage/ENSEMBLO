//! Chaque page du site, rendue par l'app elle-même à travers le client de test de
//! Paquets (sans port).

use ensemblo_site::server::app;

#[tokio::test]
async fn the_home_page_is_complete() {
    let client = app().test().await;
    let res = client.get("/").await;
    assert_eq!(res.status(), 200);
    let csp = res
        .header("content-security-policy")
        .expect("a Content-Security-Policy")
        .to_owned();
    assert!(csp.contains("frame-ancestors 'none'"));
    let html = res.text();
    assert!(html.starts_with("<!DOCTYPE html><html lang=\"fr\">"));
    assert!(html.contains("<title>Ensemblo · On fait quoi ce soir ?</title>"));
    assert!(html.contains(r#"<meta property="og:image""#));
    assert!(html.contains("application/ld+json"));
    // Les quatre îlots, rendus par le serveur.
    for island in ["hero", "ballot", "swipe", "flap"] {
        assert!(
            html.contains(&format!(r#"data-island="{island}""#)),
            "{island}"
        );
    }
    // Les sections et leurs ancres.
    for id in [
        "concept",
        "rejoindre",
        "demo",
        "temps-reel",
        "resultat",
        "ios",
        "technique",
        "faq",
    ] {
        assert!(html.contains(&format!(r#"id="{id}""#)), "{id}");
    }
    // Lisible sans JavaScript : la démo montre déjà sa première carte.
    assert!(html.contains("Ciné : le dernier Villeneuve"));
    // Le manifeste de la PWA.
    assert!(html.contains("manifest"));
}

#[tokio::test]
async fn the_other_routes_answer() {
    let client = app().test().await;
    let legal = client.get("/mentions-legales").await;
    assert_eq!(legal.status(), 200);
    assert!(legal.text().contains("Mentions légales"));

    let missing = client.get("/nulle-part").await;
    assert_eq!(missing.status(), 404);
    assert!(missing.text().contains("On ne sait pas non plus."));

    let sitemap = client.get("/sitemap.xml").await;
    assert_eq!(sitemap.status(), 200);
    assert!(sitemap.text().contains("/mentions-legales</loc>"));

    let robots = client.get("/robots.txt").await;
    assert!(robots.text().contains("Sitemap: "));

    assert_eq!(client.get("/health").await.status(), 200);
    assert_eq!(client.get("/site.css").await.status(), 200);
}
