/// Le décompte d'une session : les idées classées par « oui », et le gagnant.
///
/// L'idée qui a le plus de « oui » gagne. En cas d'égalité, le gagnant est tiré au sort
/// parmi les ex æquo avec la graine de la session : chaque appareil fait le même tirage
/// et affiche le même gagnant, sans se concerter.
public struct Tally: Hashable, Sendable {
    /// Une ligne du classement.
    public struct Row: Hashable, Sendable, Identifiable {
        public var idea: Idea
        public var yes: Int
        public var no: Int
        public var id: String { idea.id }
    }

    /// Les idées, de la plus aimée à la moins aimée (à égalité, dans l'ordre où elles ont
    /// été proposées).
    public let ranking: [Row]
    /// Les idées en tête : plusieurs en cas d'égalité, aucune sans idée ou sans « oui ».
    public let leaders: [Idea]
    /// Le gagnant : le seul en tête, ou celui du tirage au sort.
    public let winner: Idea?

    public init(ideas: [Idea], seed: UInt64) {
        let ordered = ideas.enumerated().map { (order: $0.offset, idea: $0.element) }
        ranking = ordered
            .sorted { a, b in
                a.idea.yes != b.idea.yes ? a.idea.yes > b.idea.yes : a.order < b.order
            }
            .map { Row(idea: $0.idea, yes: $0.idea.yes, no: $0.idea.no) }
        let best = ideas.map(\.yes).max() ?? 0
        leaders = best > 0 ? ideas.filter { $0.yes == best } : []
        winner = Tally.draw(among: leaders, seed: seed)
    }

    /// Plusieurs idées sont à égalité en tête.
    public var isTie: Bool { leaders.count > 1 }

    /// Le plus grand nombre de « oui » (pour la longueur des barres).
    public var maxYes: Int { ranking.first?.yes ?? 0 }

    /// Le tirage : `leaders[mix(seed) % count]`, le même calcul que sur le site web.
    public static func draw<T>(among leaders: [T], seed: UInt64) -> T? {
        guard !leaders.isEmpty else { return nil }
        return leaders[Int(mix(seed) % UInt64(leaders.count))]
    }

    /// SplitMix64 : une graine donne toujours le même nombre, sur toutes les plateformes.
    public static func mix(_ seed: UInt64) -> UInt64 {
        var z = seed &+ 0x9E37_79B9_7F4A_7C15
        z = (z ^ (z >> 30)) &* 0xBF58_476D_1CE4_E5B9
        z = (z ^ (z >> 27)) &* 0x94D0_49BB_1331_11EB
        return z ^ (z >> 31)
    }
}
