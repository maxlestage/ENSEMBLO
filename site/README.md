# Le site d'Ensemblo

Le site de présentation d'Ensemblo, écrit **en Rust avec Paquets** : le serveur est une
`paquets::App`, les pages sont des `paquets::Page` (HTML fini, balises SEO, Open Graph,
JSON‑LD, Content‑Security‑Policy stricte), les parties interactives sont des îlots Active
compilés en WebAssembly. Mobile first, aucun défilement horizontal de 320 px à l'écran large.

## Ce qu'il y a dedans

| Partie | Où |
|---|---|
| Serveur, pages, plan du site, PWA | `src/server.rs` (natif) |
| Animations 2D avec physique (disques, gravité, chocs) | `src/sim.rs` (serveur et navigateur), dessinées par `src/client/canvas.rs` |
| Îlots : scène d'accueil, urnes, démo de swipe, code de session | `src/islands.rs` |
| Export statique pour GitHub Pages | `src/export.rs` |
| Animations du navigateur (apparitions, compteurs, glissement à la souris) | `src/client/` (wasm32) |
| Textes, idées, décompte et tirage au sort | `src/content.rs` |
| Styles, icône, image Open Graph | `public/` |

- **Accueil** : six cartes d'idées flottent au-dessus d'un plateau ; des jetons « oui » et
  « non » tombent, rebondissent et s'empilent (gravité, chocs, frottement). Un appui en
  lâche un de plus.
- **Démo** : une pile de cartes à glisser au doigt (tampons OUI / NON), les votes de trois
  amis, le classement animé et, en cas d'égalité, une roulette qui tire le gagnant.
- **Résultat** : une bille par « oui » tombe dans l'urne de son idée ; la plus remplie reçoit
  sa couronne.
- **En CSS** : titre à mots défilants, bandeau, cartes en orbite, ticket au code dont les
  lettres tombent, téléphones et iPhone qui se balancent, parallaxe pilotée par le
  défilement (`animation-timeline: view()`), FAQ qui s'ouvre en douceur (`::details-content`).
- Tout est en 2D : pas de WebGL, un bundle WebAssembly d'environ 230 Ko. Les animations ne
  tournent que lorsqu'elles sont à l'écran ; tout respecte `prefers-reduced-motion`. Sans
  JavaScript, la page reste complète.

## Construire et lancer

Depuis ce dossier (Paquets sert `public/` et `pkg/` depuis le dossier courant) :

```bash
# 1. Le bundle du navigateur → pkg/
cargo build --lib --target wasm32-unknown-unknown --profile wasm-release
wasm-bindgen --target web --no-typescript --remove-name-section --remove-producers-section \
  --out-dir pkg target/wasm32-unknown-unknown/wasm-release/ensemblo_site.wasm

# 2. Le serveur, sur http://localhost:3000 (PORT sur Heroku)
cargo run --release
```

`wasm-bindgen-cli` doit avoir la version de `Cargo.toml` (`0.2.130`) :
`cargo install wasm-bindgen-cli --version 0.2.130 --locked`.

Variables : `PORT`, `SITE_URL` (adresse publique des URL canoniques et Open Graph ; prise de
la requête sinon).

## GitHub Pages

```bash
BASE_PATH=/ENSEMBLO SITE_URL=https://maxlestage.github.io/ENSEMBLO \
  cargo run --release -- export dist
```

Chaque page est rendue par l'app elle-même (`app.test()`), puis écrite dans
`dist/<chemin>/index.html`, avec `404.html`, le manifeste et le service worker de la PWA,
`sitemap.xml`, `robots.txt`, `public/` et `pkg/`. Chaque URL absolue passe derrière
`BASE_PATH`. Construisez `pkg/` d'abord. Le workflow `.github/workflows/pages.yml` fait tout
cela et publie le résultat.

## Vérifier

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo clippy --lib --target wasm32-unknown-unknown -- -D warnings
cargo test
```

Les tests couvrent le décompte et le tirage, le rendu des îlots, chaque page à travers
`app.test()` (statut, CSP, balises, sections), les URL de l'export, et la physique des deux
animations, avancées pas à pas sans navigateur : chaque jeton se pose sur le plateau sans
chevaucher les autres, chaque bille tombe dans son urne et la couronne apparaît.
