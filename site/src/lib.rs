//! Le site de présentation d'Ensemblo, écrit avec Paquets : les textes, les îlots et les
//! scènes 3D partagés par le serveur et le navigateur, le serveur (natif) et le bundle du
//! navigateur (wasm32).

pub mod content;
pub mod icons;
pub mod islands;
pub mod scenes;

#[cfg(not(target_arch = "wasm32"))]
pub mod server;

#[cfg(target_arch = "wasm32")]
mod client;
