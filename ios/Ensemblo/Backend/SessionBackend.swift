import EnsembloCore
import Foundation

/// Là où vivent les sessions : Firebase Realtime Database en vrai, la mémoire de
/// l'appareil pour les aperçus SwiftUI et quand Firebase n'est pas configuré.
protocol SessionBackend: Sendable {
    /// L'identifiant de cet appareil (connexion anonyme).
    func signIn() async throws -> String
    func create(_ session: Session) async throws
    /// Ajoute `participant` à la session `code` ; `false` si elle n'existe pas ou a expiré.
    func join(code: String, as participant: Participant) async throws -> Bool
    func add(_ idea: Idea, to code: String) async throws
    func vote(_ yes: Bool, on ideaId: String, in code: String, by userId: String) async throws
    func delete(code: String) async throws
    /// La session, à chaque changement, tant que la séquence est écoutée.
    func observe(code: String) -> AsyncStream<Session?>
    /// Un nouvel identifiant d'idée.
    func newIdeaId(in code: String) -> String
}

enum BackendError: LocalizedError {
    case notSignedIn

    var errorDescription: String? {
        switch self {
        case .notSignedIn: "Connexion impossible pour le moment."
        }
    }
}

/// Les sessions en mémoire : rien ne quitte l'appareil. Pour les aperçus, les tests et
/// l'app sans `GoogleService-Info.plist`.
actor LocalBackend: SessionBackend {
    private var sessions: [String: Session] = [:]
    private var watchers: [String: [UUID: AsyncStream<Session?>.Continuation]] = [:]
    private let userId = "local-\(UUID().uuidString.prefix(8))"

    init(seed sessions: [Session] = []) {
        for session in sessions {
            self.sessions[session.id] = session
        }
    }

    func signIn() async throws -> String { userId }

    func create(_ session: Session) async throws {
        sessions[session.id] = session
        publish(session.id)
    }

    func join(code: String, as participant: Participant) async throws -> Bool {
        guard var session = sessions[code], !session.isExpired() else { return false }
        if !session.participants.contains(where: { $0.id == participant.id }) {
            session.participants.append(participant)
        }
        sessions[code] = session
        publish(code)
        return true
    }

    func add(_ idea: Idea, to code: String) async throws {
        sessions[code]?.ideas.append(idea)
        publish(code)
    }

    func vote(_ yes: Bool, on ideaId: String, in code: String, by userId: String) async throws {
        guard let index = sessions[code]?.ideas.firstIndex(where: { $0.id == ideaId }) else { return }
        sessions[code]?.ideas[index].votes[userId] = yes
        publish(code)
    }

    func delete(code: String) async throws {
        sessions[code] = nil
        publish(code)
    }

    nonisolated func observe(code: String) -> AsyncStream<Session?> {
        AsyncStream { continuation in
            let id = UUID()
            Task { await self.watch(code: code, id: id, continuation: continuation) }
            continuation.onTermination = { _ in
                Task { await self.unwatch(code: code, id: id) }
            }
        }
    }

    nonisolated func newIdeaId(in code: String) -> String {
        "idea-\(UUID().uuidString)"
    }

    private func watch(code: String, id: UUID, continuation: AsyncStream<Session?>.Continuation) {
        watchers[code, default: [:]][id] = continuation
        continuation.yield(sessions[code])
    }

    private func unwatch(code: String, id: UUID) {
        watchers[code]?[id] = nil
    }

    private func publish(_ code: String) {
        for continuation in watchers[code, default: [:]].values {
            continuation.yield(sessions[code])
        }
    }
}
