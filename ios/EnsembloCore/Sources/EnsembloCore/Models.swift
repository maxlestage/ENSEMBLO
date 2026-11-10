import Foundation

/// Une personne dans une session. L'identifiant vient de la connexion anonyme Firebase.
public struct Participant: Hashable, Sendable, Identifiable, Codable {
    public var id: String
    public var name: String

    public init(id: String, name: String) {
        self.id = id
        self.name = name
    }
}

/// Une idée de sortie, et les votes qu'elle a reçus (`userId` → oui / non).
public struct Idea: Hashable, Sendable, Identifiable, Codable {
    public var id: String
    public var text: String
    public var createdBy: String
    public var createdAt: Date
    public var votes: [String: Bool]

    public init(
        id: String,
        text: String,
        createdBy: String,
        createdAt: Date = .now,
        votes: [String: Bool] = [:]
    ) {
        self.id = id
        self.text = text
        self.createdBy = createdBy
        self.createdAt = createdAt
        self.votes = votes
    }

    /// Le nombre de « oui ».
    public var yes: Int { votes.values.count(where: { $0 }) }

    /// Le nombre de « non ».
    public var no: Int { votes.values.count(where: { !$0 }) }

    /// Le vote de `userId`, s'il a voté.
    public func vote(of userId: String) -> Bool? { votes[userId] }
}

/// Une session : son créateur, ses participants, ses idées et la graine du tirage au sort.
///
/// Dans Firebase Realtime Database :
///
/// ```text
/// sessions/{code}
/// ├─ createdAt     1731276000000 (ms)
/// ├─ creator       { id, name }
/// ├─ participants  { userId: name }
/// ├─ seed          2718281828
/// └─ items/{itemId}
///    ├─ text, createdBy, createdAt
///    └─ votes      { userId: true | false }
/// ```
public struct Session: Hashable, Sendable, Identifiable {
    /// Le code à six caractères, qui est aussi la clé de la session.
    public var id: String
    public var createdAt: Date
    public var creator: Participant
    public var participants: [Participant]
    public var ideas: [Idea]
    /// La graine du tirage : la même sur chaque appareil, donc le même gagnant.
    public var seed: UInt64

    public init(
        id: String,
        createdAt: Date = .now,
        creator: Participant,
        participants: [Participant] = [],
        ideas: [Idea] = [],
        seed: UInt64
    ) {
        self.id = id
        self.createdAt = createdAt
        self.creator = creator
        self.participants = participants
        self.ideas = ideas
        self.seed = seed
    }

    /// Une session dure 24 heures.
    public static let lifetime: TimeInterval = 24 * 60 * 60

    public func isExpired(at now: Date = .now) -> Bool {
        now.timeIntervalSince(createdAt) > Self.lifetime
    }

    /// Les idées que `userId` n'a pas encore jugées, dans l'ordre où elles ont été proposées.
    public func pending(for userId: String) -> [Idea] {
        ideas.filter { $0.vote(of: userId) == nil }
    }

    /// `userId` a voté pour chaque idée.
    public func hasFinished(_ userId: String) -> Bool {
        !ideas.isEmpty && pending(for: userId).isEmpty
    }

    /// Les participants qui ont fini de voter.
    public var finished: [Participant] {
        participants.filter { hasFinished($0.id) }
    }

    /// Le résultat, tel qu'il est maintenant.
    public var result: Tally { Tally(ideas: ideas, seed: seed) }
}
