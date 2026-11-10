import Foundation

/// La traduction entre une session et l'arbre JSON de Firebase Realtime Database
/// (`DataSnapshot.value`, un dictionnaire `[String: Any]`).
public enum Snapshot {
    /// Une session depuis la valeur du nœud `sessions/{code}` ; `nil` si elle n'existe pas
    /// ou n'a pas de créateur.
    public static func session(code: String, value: Any?) -> Session? {
        guard let root = value as? [String: Any],
              let creator = root["creator"] as? [String: Any],
              let creatorId = creator["id"] as? String
        else { return nil }
        let creatorName = creator["name"] as? String ?? "?"
        var participants = (root["participants"] as? [String: Any] ?? [:])
            .compactMap { id, name in (name as? String).map { Participant(id: id, name: $0) } }
            .sorted { $0.name.localizedStandardCompare($1.name) == .orderedAscending }
        if !participants.contains(where: { $0.id == creatorId }) {
            participants.insert(Participant(id: creatorId, name: creatorName), at: 0)
        }
        let ideas = (root["items"] as? [String: Any] ?? [:])
            .compactMap { id, value in idea(id: id, value: value) }
            .sorted { ($0.createdAt, $0.id) < ($1.createdAt, $1.id) }
        return Session(
            id: code,
            createdAt: date(root["createdAt"]) ?? .distantPast,
            creator: Participant(id: creatorId, name: creatorName),
            participants: participants,
            ideas: ideas,
            seed: number(root["seed"]).map { UInt64(truncatingIfNeeded: $0) } ?? 0
        )
    }

    static func idea(id: String, value: Any) -> Idea? {
        guard let item = value as? [String: Any], let text = item["text"] as? String else {
            return nil
        }
        let votes = (item["votes"] as? [String: Any] ?? [:]).compactMapValues { $0 as? Bool }
        return Idea(
            id: id,
            text: text,
            createdBy: item["createdBy"] as? String ?? "",
            createdAt: date(item["createdAt"]) ?? .distantPast,
            votes: votes
        )
    }

    /// Le nœud d'une nouvelle session.
    public static func value(of session: Session) -> [String: Any] {
        [
            "createdAt": milliseconds(session.createdAt),
            "creator": ["id": session.creator.id, "name": session.creator.name],
            "participants": Dictionary(
                uniqueKeysWithValues: session.participants.map { ($0.id, $0.name) }
            ),
            // Les nombres de Firebase sont des doubles : la graine reste sous 2^53.
            "seed": Int64(session.seed & ((1 << 53) - 1)),
        ]
    }

    /// Le nœud d'une nouvelle idée.
    public static func value(of idea: Idea) -> [String: Any] {
        [
            "text": idea.text,
            "createdBy": idea.createdBy,
            "createdAt": milliseconds(idea.createdAt),
        ]
    }

    public static func milliseconds(_ date: Date) -> Int64 {
        Int64((date.timeIntervalSince1970 * 1000).rounded())
    }

    static func number(_ value: Any?) -> Int64? {
        switch value {
        case let n as Int64:
            return n
        case let n as Int:
            return Int64(n)
        case let n as Double:
            return Int64(exactly: n.rounded())
        case let n as NSNumber:
            return n.int64Value
        default:
            return nil
        }
    }

    static func date(_ value: Any?) -> Date? {
        number(value).map { Date(timeIntervalSince1970: Double($0) / 1000) }
    }
}
