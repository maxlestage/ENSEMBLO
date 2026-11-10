import EnsembloCore
@preconcurrency import FirebaseAuth
@preconcurrency import FirebaseDatabase
import Foundation

/// Les sessions dans Firebase Realtime Database, sous `sessions/{code}`. Chaque appareil
/// se connecte anonymement : les règles (`firebase/database.rules.json`) n'acceptent que
/// les utilisateurs connectés, et chacun n'écrit que ses propres votes.
final class FirebaseBackend: SessionBackend, @unchecked Sendable {
    // `DatabaseReference` est sûr entre les fils (le SDK sérialise ses accès).
    private let sessions: DatabaseReference

    init() {
        let database = Database.database()
        // Les votes faits hors ligne partent au retour du réseau.
        database.isPersistenceEnabled = true
        sessions = database.reference(withPath: "sessions")
    }

    func signIn() async throws -> String {
        if let user = Auth.auth().currentUser {
            return user.uid
        }
        return try await Auth.auth().signInAnonymously().user.uid
    }

    func create(_ session: Session) async throws {
        _ = try await sessions.child(session.id).setValue(Snapshot.value(of: session))
    }

    func join(code: String, as participant: Participant) async throws -> Bool {
        let node = sessions.child(code)
        // Une session absente ou expirée n'est pas lisible : les règles refusent.
        guard let snapshot = try? await node.child("createdAt").getData(),
              snapshot.exists(),
              let createdAt = snapshot.value as? NSNumber,
              Date.now.timeIntervalSince1970 * 1000 - createdAt.doubleValue < Session.lifetime * 1000
        else { return false }
        _ = try await node.child("participants").child(participant.id).setValue(participant.name)
        node.keepSynced(true)
        return true
    }

    func add(_ idea: Idea, to code: String) async throws {
        _ = try await sessions.child(code).child("items").child(idea.id).setValue(Snapshot.value(of: idea))
    }

    func vote(_ yes: Bool, on ideaId: String, in code: String, by userId: String) async throws {
        _ = try await sessions.child(code).child("items").child(ideaId).child("votes").child(userId)
            .setValue(yes)
    }

    func delete(code: String) async throws {
        _ = try await sessions.child(code).removeValue()
    }

    func observe(code: String) -> AsyncStream<Session?> {
        let node = sessions.child(code)
        return AsyncStream { continuation in
            let handle = node.observe(.value) { snapshot in
                continuation.yield(Snapshot.session(code: code, value: snapshot.value))
            } withCancel: { _ in
                // Session expirée ou supprimée : les règles refusent la lecture.
                continuation.yield(nil)
                continuation.finish()
            }
            continuation.onTermination = { _ in
                node.removeObserver(withHandle: handle)
            }
        }
    }

    func newIdeaId(in code: String) -> String {
        sessions.child(code).child("items").childByAutoId().key ?? UUID().uuidString
    }
}
