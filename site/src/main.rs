//! `cargo run` : le site sur http://localhost:3000 (`PORT` sur Heroku). Lancez-le depuis
//! `site/`, après avoir construit le bundle du navigateur (voir `README.md`).
//!
//! `cargo run -- export dist` : le site entier en fichiers statiques dans `dist/`, pour
//! GitHub Pages. `BASE_PATH` (ex. `/ENSEMBLO`) préfixe chaque lien, `SITE_URL`
//! (ex. `https://maxlestage.github.io/ENSEMBLO`) donne les URL absolues.

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("export") => {
            let dir = args.next().unwrap_or_else(|| "dist".into());
            let base = std::env::var("BASE_PATH").unwrap_or_default();
            let site_url = std::env::var("SITE_URL")
                .ok()
                .filter(|u| !u.trim().is_empty())
                .unwrap_or_else(|| format!("http://localhost:3000{base}"));
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;
            let files = runtime.block_on(ensemblo_site::export::export(
                std::path::Path::new(&dir),
                &base,
                &site_url,
            ))?;
            println!("{files} fichiers écrits dans {dir}");
            Ok(())
        }
        Some(other) => {
            eprintln!("commande inconnue {other:?} : lancez sans argument, ou `export <dossier>`");
            std::process::exit(2);
        }
        None => ensemblo_site::server::run(),
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
