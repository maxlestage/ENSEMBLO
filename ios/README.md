# Ensemblo pour iPhone

L'app native d'Ensemblo, en **Swift 6** et **SwiftUI** (Liquid Glass d'iOS 26), avec le
**SDK Firebase pour Apple 13** (Realtime Database et connexion anonyme).

- Créer une session : un code à six caractères, son QR‑code et son lien de partage
  (`ShareLink`).
- Rejoindre : taper le code, scanner le QR‑code (VisionKit), ou ouvrir un lien
  `ensemblo://s/CODE`.
- Proposer des idées, qui apparaissent aussitôt chez tout le monde.
- Voter en swipant les cartes (tampons OUI / NON, retour haptique), ou avec les boutons
  et VoiceOver.
- Le résultat en direct ; en cas d'égalité, une roulette tire le gagnant avec la graine de
  la session : le même sur chaque téléphone, et le même calcul que sur le site.
- Hors ligne : Firebase garde la session en cache et envoie les votes au retour du réseau.

## Ouvrir le projet

Prérequis : Xcode 27 (SDK iOS 27, Swift 6.4), [XcodeGen](https://github.com/yonaskolb/XcodeGen) 2.46+.

```bash
brew install xcodegen
cd ios
xcodegen generate        # crée Ensemblo.xcodeproj depuis project.yml
open Ensemblo.xcodeproj
```

Choisissez votre équipe dans *Signing & Capabilities* (ou `DEVELOPMENT_TEAM` dans
`project.yml`) pour installer l'app sur un iPhone. Cible minimale : iOS 26.

## Brancher Firebase

Sans configuration, l'app tourne en **mode local** : tout fonctionne, mais les sessions
restent sur l'appareil. Pour le temps réel entre téléphones :

1. Dans la [console Firebase](https://console.firebase.google.com/), créez un projet, puis
   une **Realtime Database**.
2. *Authentication* → *Sign-in method* → activez **Anonyme**.
3. *Realtime Database* → *Règles* : collez le contenu de
   [`../firebase/database.rules.json`](../firebase/database.rules.json), puis publiez.
4. *Paramètres du projet* → *Ajouter une app* → **iOS**, identifiant `app.ensemblo.ios`.
   Téléchargez `GoogleService-Info.plist` et posez‑le dans `ios/` (il n'est pas versionné) :
   une phase de script le copie dans l'app à chaque compilation.
5. Recompilez.

Pour que le QR‑code ouvre aussi une page web chez les amis sans l'app, mettez l'adresse de
cette page dans `EnsembloWebURL` (`project.yml`) : les liens deviennent
`https://…/s/CODE`. Vide, ils restent en `ensemblo://s/CODE`.

### Les règles

- Seuls les utilisateurs connectés (anonymement) lisent et écrivent.
- Une session n'est lisible que pendant 24 heures ; l'app de son créateur l'efface ensuite.
- Le créateur seul crée ou supprime sa session ; chacun n'ajoute que son prénom, ses idées
  (120 caractères au plus) et ses propres votes, une fois inscrit dans la session.

## Organisation

| Dossier | Quoi |
|---|---|
| `EnsembloCore/` | Paquet Swift sans interface : modèles, décompte, tirage, codes et liens, lecture des données Firebase. Testé avec Swift Testing (`swift test`, macOS et Linux). |
| `Ensemblo/App` | Le point d'entrée, les liens `ensemblo://` |
| `Ensemblo/Backend` | `SessionStore` (état observable), Firebase, et le mode local |
| `Ensemblo/Views` | Accueil, session, idées, vote, résultat, QR‑code et scanner |
| `project.yml` | Le projet Xcode (XcodeGen) |

```bash
swift test --package-path EnsembloCore
```
