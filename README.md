<div align="center">

<img src="site/public/og-image.png" alt="Ensemblo : On fait quoi ce soir ?" width="720"/>

# Ensemblo

**« On fait quoi ce soir ? »** Une application de décision de groupe : on crée une session,
on partage le QR‑code, chacun propose ses idées et vote en swipant. L'idée qui a le plus de
« oui » gagne ; un tirage au sort départage les égalités, le même sur tous les téléphones.

</div>

| Dossier | Quoi | Avec |
|---|---|---|
| [`site/`](site/) | Le site de présentation : animations 2D avec physique, démo de vote jouable | Rust, [Paquets](https://github.com/maxlestage/Paquets) (pages rendues par le serveur, îlots WebAssembly, canvas 2D) |
| [`ios/`](ios/) | L'app iPhone | Swift 6, SwiftUI (Liquid Glass), Firebase Apple SDK 13 |
| [`firebase/`](firebase/) | Les règles de sécurité de la base | Firebase Realtime Database |

## Le site

```bash
cd site
cargo build --lib --target wasm32-unknown-unknown --profile wasm-release
wasm-bindgen --target web --no-typescript --out-dir pkg \
  target/wasm32-unknown-unknown/wasm-release/ensemblo_site.wasm
cargo run --release            # http://localhost:3000
```

Détails dans [`site/README.md`](site/README.md).

## L'app iOS

```bash
brew install xcodegen
cd ios && xcodegen generate && open Ensemblo.xcodeproj
```

Sans configuration Firebase, l'app tourne en **mode local** (les sessions restent sur
l'appareil). Pour le temps réel, voir [`ios/README.md`](ios/README.md).

## Le modèle de données

```text
sessions/{code}                 code à 6 caractères (A–Z, 2–9)
├─ createdAt     1731276000000  ms ; la session expire au bout de 24 h
├─ creator       { id, name }
├─ participants  { userId: name }
├─ seed          2718281828     la graine du tirage au sort, partagée
└─ items/{itemId}
   ├─ text, createdBy, createdAt
   └─ votes      { userId: true | false }
```

Le gagnant : l'idée qui a le plus de « oui ». À égalité, `leaders[splitmix64(seed) % n]`,
calculé à l'identique en Rust (site) et en Swift (app), et vérifié par les tests des deux.

## Intégration continue

- `.github/workflows/ci.yml` : site (fmt, clippy serveur et wasm32, tests, bundle) et tests
  Swift du cœur sous Linux ; quand tout est vert, la branche est **fusionnée
  automatiquement dans `master`** (sauf `wip/...`), puis publiée.
- `.github/workflows/pages.yml` : le site exporté en fichiers statiques et publié sur
  **GitHub Pages** (`https://maxlestage.github.io/ENSEMBLO/`), à chaque mise à jour de
  `master`. À activer une fois : *Settings → Pages → Source : GitHub Actions*.
- `.github/workflows/ios.yml` : l'app compilée avec Xcode 27 (runner `xcode-27`), quand
  `ios/` change.
- `.github/workflows/heroku.yml` : déploiement du site sur Heroku (secrets
  `HEROKU_API_KEY`, `REPOS_TOKEN`, variable `HEROKU_APP_NAME`).

Le secret `REPOS_TOKEN` (un jeton GitHub qui peut lire les dépôts de Paquets, privés) est
nécessaire à la compilation du site.

Pour la publication sur GitHub Pages en local :

```bash
cd site
BASE_PATH=/ENSEMBLO SITE_URL=https://maxlestage.github.io/ENSEMBLO cargo run --release -- export dist
```
