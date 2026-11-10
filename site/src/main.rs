//! `cargo run` : le site sur http://localhost:3000 (`PORT` sur Heroku). Lancez-le depuis
//! `site/`, après avoir construit le bundle du navigateur (voir `README.md`).

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::io::Result<()> {
    ensemblo_site::server::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
