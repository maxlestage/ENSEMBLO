//! `ensemblo-site export <dossier>` : le site entier en fichiers statiques, pour GitHub
//! Pages ou tout hébergeur statique. Chaque page vient du serveur lui-même, à travers le
//! client de test de Paquets (`app.test()`) : les deux sont identiques.
//!
//! Sur GitHub Pages, le site vit sous un chemin (`/ENSEMBLO`) : `BASE_PATH` préfixe
//! chaque URL absolue des pages, du manifeste et du service worker.

use std::io;
use std::path::{Path, PathBuf};

use crate::server::{app, set_site_url, site_dir};

/// Les pages du site, et leur fichier.
pub const PAGES: [(&str, &str); 2] = [
    ("/", "index.html"),
    ("/mentions-legales", "mentions-legales/index.html"),
];

/// Écrit le site dans `out`, liens derrière `base` (`""` ou `/ENSEMBLO`), URL absolues
/// (canoniques, Open Graph, plan du site) sur `site_url`. Rend le nombre de fichiers.
pub async fn export(out: &Path, base: &str, site_url: &str) -> io::Result<usize> {
    let base = base.trim_end_matches('/');
    let out = std::path::absolute(out)?;
    // Paquets trouve `public/` et `pkg/` dans le dossier courant.
    let dir = std::path::absolute(site_dir())?;
    std::env::set_current_dir(&dir)?;
    if !dir.join("pkg").is_dir() {
        eprintln!(
            "attention : site/pkg manque, les pages ne seront pas interactives (voir site/README.md)"
        );
    }
    set_site_url(site_url);
    let mut app = app();
    // Des fichiers statiques n'ont pas de serveur à protéger, et l'export demande
    // toutes les pages d'un coup.
    app.security(None);
    let client = app.test().await;

    let mut written = 0;
    let mut write = |path: &str, bytes: &[u8]| -> io::Result<()> {
        let file = out.join(path.trim_start_matches('/'));
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(file, bytes)?;
        written += 1;
        Ok(())
    };
    let get = |path: &'static str, expected: u16| {
        let client = &client;
        async move {
            let res = client.get(path).await;
            if res.status().as_u16() != expected {
                return Err(io::Error::other(format!(
                    "GET {path} : {} au lieu de {expected}",
                    res.status()
                )));
            }
            Ok(res.text())
        }
    };

    for (path, file) in PAGES {
        write(file, with_base(&get(path, 200).await?, base).as_bytes())?;
    }
    // La page « introuvable » : GitHub Pages sert 404.html pour toute adresse inconnue.
    write(
        "404.html",
        with_base(&get("/404", 404).await?, base).as_bytes(),
    )?;
    // La PWA : le manifeste et le service worker listent des URL absolues.
    for file in ["/manifest.webmanifest", "/sw.js"] {
        write(
            file,
            with_base_everywhere(&get(file, 200).await?, base).as_bytes(),
        )?;
    }
    for file in ["/sitemap.xml", "/robots.txt"] {
        write(file, get(file, 200).await?.as_bytes())?;
    }
    for folder in ["public", "pkg"] {
        let root = dir.join(folder);
        let prefix = if folder == "pkg" { "pkg/" } else { "" };
        for file in files(&root) {
            let relative = file.strip_prefix(&root).map_err(io::Error::other)?;
            let url = format!("{prefix}{}", relative.to_string_lossy().replace('\\', "/"));
            // Les fichiers de wasm-bindgen qui ne servent pas au bundle.
            if url.ends_with(".gitignore") || url.ends_with("package.json") {
                continue;
            }
            write(&url, &std::fs::read(&file)?)?;
        }
    }
    // Les fichiers sont servis tels quels : pas de Jekyll sur GitHub Pages.
    write(".nojekyll", b"")?;
    Ok(written)
}

/// Les URL absolues d'une page HTML (`href="/…"`, `src="/…"`, le chargeur des îlots,
/// l'enregistrement du service worker) derrière `base`.
pub fn with_base(html: &str, base: &str) -> String {
    if base.is_empty() {
        return html.to_owned();
    }
    html.replace("=\"/", &format!("=\"{base}/"))
        .replace("\"/pkg/", &format!("\"{base}/pkg/"))
        .replace("\"/sw.js\"", &format!("\"{base}/sw.js\""))
}

/// Chaque URL absolue (`"/…"`) d'un fichier JSON ou JavaScript derrière `base`.
pub fn with_base_everywhere(text: &str, base: &str) -> String {
    if base.is_empty() {
        return text.to_owned();
    }
    text.replace("\"/", &format!("\"{base}/"))
}

/// Chaque fichier sous `dir`, récursivement.
fn files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(files(&path));
        } else {
            found.push(path);
        }
    }
    found.sort();
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes_every_absolute_url() {
        let html = r#"<link href="/site.css"/><a href="/#demo">x</a><a href="https://exemple.fr/">y</a><script type="module">import init from "/pkg/a.js";init({module_or_path:"/pkg/a_bg.wasm"});navigator.serviceWorker.register("/sw.js")</script>"#;
        let out = with_base(html, "/ENSEMBLO");
        assert!(out.contains(r#"href="/ENSEMBLO/site.css""#));
        assert!(out.contains(r#"href="/ENSEMBLO/#demo""#));
        assert!(out.contains(r#"href="https://exemple.fr/""#));
        assert_eq!(out.matches("\"/ENSEMBLO/pkg/").count(), 2);
        assert!(out.contains("\"/ENSEMBLO/sw.js\""));
        assert!(!out.contains("/ENSEMBLO/ENSEMBLO"));
        assert_eq!(with_base(html, ""), html);
        assert_eq!(
            with_base_everywhere(r#"{"start_url":"/","src":"/icons/a.png"}"#, "/E"),
            r#"{"start_url":"/E/","src":"/E/icons/a.png"}"#
        );
    }
}
