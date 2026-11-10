//! Les textes et les données du site, partagés par le serveur et le navigateur.

/// Une idée de sortie, comme dans l'application.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Idea {
    pub emoji: &'static str,
    pub title: &'static str,
    pub kind: &'static str,
    pub detail: &'static str,
    /// La teinte de la carte (degrés HSL).
    pub hue: u16,
    /// Proposée par…
    pub by: &'static str,
}

/// Les idées de la démo.
pub const IDEAS: [Idea; 6] = [
    Idea {
        emoji: "🎬",
        title: "Ciné : le dernier Villeneuve",
        kind: "Film",
        detail: "Séance de 21 h · 2 h 46",
        hue: 262,
        by: "Alice",
    },
    Idea {
        emoji: "🍜",
        title: "Ramen chez Kodawari",
        kind: "Restaurant",
        detail: "À 12 min à pied · 18 €",
        hue: 18,
        by: "Bastien",
    },
    Idea {
        emoji: "🎲",
        title: "Soirée jeux de société",
        kind: "À la maison",
        detail: "Chez Chloé · apporter des chips",
        hue: 152,
        by: "Chloé",
    },
    Idea {
        emoji: "🎤",
        title: "Karaoké au Bunny's",
        kind: "Sortie",
        detail: "Salle privée · jusqu'à 2 h",
        hue: 322,
        by: "Alice",
    },
    Idea {
        emoji: "🌉",
        title: "Balade sur les quais",
        kind: "Plein air",
        detail: "Coucher du soleil à 19 h 42",
        hue: 196,
        by: "Bastien",
    },
    Idea {
        emoji: "🍕",
        title: "Pizza & série",
        kind: "Canapé",
        detail: "Livraison en 30 min",
        hue: 42,
        by: "Chloé",
    },
];

/// Les autres participants de la démo, et leurs votes (dans l'ordre de [`IDEAS`]).
pub const FRIENDS: [(&str, [bool; 6]); 3] = [
    ("Alice", [true, true, false, true, false, true]),
    ("Bastien", [false, true, true, false, true, true]),
    ("Chloé", [true, false, true, true, false, false]),
];

/// Les cinq étapes, de la création au résultat.
pub const STEPS: [(&str, &str, &str); 5] = [
    (
        "spark",
        "Créer",
        "Un appui crée la session. Elle a son lien, son code à six caractères et son QR‑code.",
    ),
    (
        "qr",
        "Rejoindre",
        "Les amis scannent le QR‑code ou ouvrent le lien, depuis un iPhone ou n'importe quel navigateur. Pas de compte.",
    ),
    (
        "bulb",
        "Suggérer",
        "Film, resto, activité : chacun ajoute ses idées, qui apparaissent aussitôt chez tout le monde.",
    ),
    (
        "swipe",
        "Voter",
        "À droite c'est oui, à gauche c'est non. Une carte après l'autre, comme un jeu.",
    ),
    (
        "crown",
        "Décider",
        "L'idée qui a le plus de oui gagne. Égalité ? Un tirage au sort tranche, en direct.",
    ),
];

/// Les points forts, avec leur icône.
pub const FEATURES: [(&str, &str, &str); 6] = [
    (
        "bolt",
        "En temps réel",
        "Firebase Realtime Database synchronise idées et votes en quelques millisecondes.",
    ),
    (
        "lock",
        "Votes discrets",
        "Pendant le vote, l'app montre qui a fini, pas ce que chacun a choisi.",
    ),
    (
        "phone",
        "iPhone et web",
        "Une app iOS native en Swift, et une PWA installable pour les autres.",
    ),
    (
        "wifi",
        "Hors ligne",
        "Les votes faits sans réseau partent dès que la connexion revient.",
    ),
    (
        "dice",
        "Tirage équitable",
        "En cas d'égalité, le hasard choisit parmi les ex æquo, et tout le monde voit le même tirage.",
    ),
    (
        "leaf",
        "Sans compte",
        "Un prénom suffit. Une session expire au bout de 24 heures.",
    ),
];

/// Les questions fréquentes.
pub const FAQ: [(&str, &str); 6] = [
    (
        "Faut‑il installer quelque chose ?",
        "Non. Celui qui crée la session peut utiliser l'app iOS ; les autres rejoignent depuis le lien, dans n'importe quel navigateur. La PWA s'installe sur l'écran d'accueil si on le souhaite.",
    ),
    (
        "Combien de personnes peuvent voter ?",
        "Autant que vous voulez : un couple, une coloc, ou les quarante personnes du week‑end d'intégration.",
    ),
    (
        "Comment l'égalité est‑elle départagée ?",
        "Parmi les idées ex æquo, une est tirée au sort. La graine du tirage est écrite dans la session : tous les appareils affichent le même gagnant.",
    ),
    (
        "Mes votes sont‑ils visibles ?",
        "Pendant le vote, l'app montre seulement qui a fini. Le classement apparaît dans le résultat.",
    ),
    (
        "Que deviennent les données ?",
        "Une session ne contient que des prénoms, des idées et des votes. Elle n'est plus lisible 24 heures après sa création, puis l'app de son créateur l'efface.",
    ),
    (
        "Ça marche sans réseau ?",
        "Oui : l'application garde la session en cache et envoie vos votes dès que le réseau revient.",
    ),
];

/// Les mots qui défilent dans le titre.
pub const ROTATING: [&str; 5] = ["un ciné", "un resto", "un karaoké", "une balade", "un jeu"];

/// Le bandeau défilant.
pub const MARQUEE: [&str; 10] = [
    "🎬 Film",
    "🍣 Sushis",
    "🎳 Bowling",
    "🧗 Escalade",
    "🎨 Expo",
    "🍻 Bar à jeux",
    "🎤 Karaoké",
    "🏓 Ping‑pong",
    "🍿 Série",
    "🌮 Tacos",
];

/// Les votes « oui » de chaque idée : les amis, plus `mine` (le visiteur, s'il a voté).
pub fn tally(mine: &[Option<bool>]) -> [usize; 6] {
    let mut counts = [0usize; 6];
    for (i, count) in counts.iter_mut().enumerate() {
        *count = FRIENDS.iter().filter(|(_, votes)| votes[i]).count()
            + usize::from(mine.get(i).copied().flatten() == Some(true));
    }
    counts
}

/// Les idées en tête (plusieurs en cas d'égalité), dans l'ordre de [`IDEAS`].
pub fn leaders(counts: &[usize]) -> Vec<usize> {
    let best = counts.iter().copied().max().unwrap_or(0);
    (0..counts.len()).filter(|&i| counts[i] == best).collect()
}

/// Le gagnant : le seul en tête, ou celui que tire `seed` parmi les ex æquo.
pub fn winner(counts: &[usize], seed: u64) -> Option<usize> {
    let leaders = leaders(counts);
    if leaders.is_empty() {
        return None;
    }
    Some(leaders[(mix(seed) % leaders.len() as u64) as usize])
}

/// SplitMix64 : la même graine donne le même tirage sur chaque appareil.
pub fn mix(seed: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_the_friends_and_the_visitor() {
        let none = [None; 6];
        assert_eq!(tally(&none), [2, 2, 2, 2, 1, 2]);
        let mut mine = [Some(false); 6];
        mine[4] = Some(true);
        assert_eq!(tally(&mine), [2, 2, 2, 2, 2, 2]);
        mine[1] = Some(true);
        assert_eq!(tally(&mine)[1], 3);
        assert_eq!(leaders(&tally(&mine)), vec![1]);
        assert_eq!(winner(&tally(&mine), 7), Some(1));
    }

    #[test]
    fn a_tie_is_drawn_among_the_leaders_only() {
        let counts = [3, 1, 3, 0, 3, 2];
        let mut seen = [false; 6];
        for seed in 0..200 {
            let w = winner(&counts, seed).unwrap();
            assert!([0, 2, 4].contains(&w));
            seen[w] = true;
        }
        // Chaque ex æquo peut gagner.
        assert!(seen[0] && seen[2] && seen[4]);
        // Le même tirage partout.
        assert_eq!(winner(&counts, 42), winner(&counts, 42));
        assert_eq!(winner(&[], 1), None);
    }
}
