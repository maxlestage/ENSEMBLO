import EnsembloCore
import Foundation
import Observation

/// L'état de l'app : qui je suis, la session ouverte, et les actions qui la modifient.
/// Les écritures partent sans attendre le serveur : Firebase met l'écran à jour tout de
/// suite (cache local) et envoie au retour du réseau.
@MainActor
@Observable
final class SessionStore {
    private(set) var userId: String?
    private(set) var session: Session?
    /// Le code de la session ouverte, même avant que ses données n'arrivent.
    private(set) var code: String?
    private(set) var isWorking = false
    var error: String?

    /// L'app tourne sans Firebase (pas de `GoogleService-Info.plist`) : tout reste local.
    let isLocal: Bool
    /// L'adresse web des liens de session (`EnsembloWebURL` dans Info.plist), si elle existe.
    let webBase: URL?

    private let backend: any SessionBackend
    private var watching: Task<Void, Never>?

    private static let createdKey = "ensemblo.created"

    init(backend: any SessionBackend, isLocal: Bool, webBase: URL? = nil) {
        self.backend = backend
        self.isLocal = isLocal
        self.webBase = webBase
    }

    var me: Participant? {
        guard let userId else { return nil }
        return session?.participants.first { $0.id == userId }
    }

    var link: URL? {
        code.map { SessionLink.url(for: $0, webBase: webBase) }
    }

    // MARK: - Connexion

    func start() async {
        guard userId == nil else { return }
        do {
            userId = try await backend.signIn()
            await cleanUpExpired()
        } catch {
            self.error = error.localizedDescription
        }
    }

    /// Les sessions créées ici et expirées sont effacées de la base.
    private func cleanUpExpired() async {
        let defaults = UserDefaults.standard
        var created = defaults.dictionary(forKey: Self.createdKey) as? [String: Double] ?? [:]
        let now = Date.now.timeIntervalSince1970
        for (code, at) in created where now - at > Session.lifetime {
            try? await backend.delete(code: code)
            created[code] = nil
        }
        defaults.set(created, forKey: Self.createdKey)
    }

    // MARK: - Sessions

    /// Crée une session et l'ouvre.
    func create(name: String) async {
        await start()
        guard let userId else { return }
        let me = Participant(id: userId, name: clean(name))
        let session = Session(
            id: SessionCode.random(),
            creator: me,
            participants: [me],
            seed: UInt64.random(in: 0..<(1 << 53))
        )
        await perform {
            try await self.backend.create(session)
            var created = UserDefaults.standard.dictionary(forKey: Self.createdKey) as? [String: Double] ?? [:]
            created[session.id] = session.createdAt.timeIntervalSince1970
            UserDefaults.standard.set(created, forKey: Self.createdKey)
            self.open(session.id)
        }
    }

    /// Rejoint la session `code` (un code tapé, ou le contenu d'un QR‑code).
    func join(_ text: String, name: String) async {
        guard let code = SessionLink.code(in: text) else {
            error = "Ce code ne ressemble pas à un code de session (six caractères)."
            return
        }
        await start()
        guard let userId else { return }
        await perform {
            let found = try await self.backend.join(
                code: code,
                as: Participant(id: userId, name: self.clean(name))
            )
            if found {
                self.open(code)
            } else {
                self.error = "Aucune session « \(code) » en cours. Elle a peut‑être expiré."
            }
        }
    }

    func leave() {
        watching?.cancel()
        watching = nil
        session = nil
        code = nil
    }

    private func open(_ code: String) {
        watching?.cancel()
        self.code = code
        let stream = backend.observe(code: code)
        watching = Task { [weak self] in
            for await session in stream {
                guard let self, !Task.isCancelled else { return }
                if let session, !session.isExpired() {
                    self.session = session
                } else if self.session != nil {
                    self.error = "La session est terminée."
                    self.leave()
                    return
                }
            }
        }
    }

    // MARK: - Idées et votes

    func add(idea text: String) {
        let text = text.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty, let code, let userId else { return }
        let idea = Idea(id: backend.newIdeaId(in: code), text: String(text.prefix(120)), createdBy: userId)
        send { try await $0.add(idea, to: code) }
    }

    func vote(_ yes: Bool, on idea: Idea) {
        guard let code, let userId else { return }
        // Le vote s'affiche tout de suite, avant même la réponse du serveur.
        if let index = session?.ideas.firstIndex(where: { $0.id == idea.id }) {
            session?.ideas[index].votes[userId] = yes
        }
        send { try await $0.vote(yes, on: idea.id, in: code, by: userId) }
    }

    // MARK: - Outils

    private func clean(_ name: String) -> String {
        let name = name.trimmingCharacters(in: .whitespacesAndNewlines)
        return name.isEmpty ? "Quelqu'un" : String(name.prefix(24))
    }

    private func perform(_ work: @MainActor () async throws -> Void) async {
        isWorking = true
        defer { isWorking = false }
        do {
            try await work()
        } catch {
            self.error = error.localizedDescription
        }
    }

    /// Une écriture qui ne bloque pas l'écran.
    private func send(_ write: @escaping @Sendable (any SessionBackend) async throws -> Void) {
        let backend = backend
        Task { [weak self] in
            do {
                try await write(backend)
            } catch {
                self?.error = error.localizedDescription
            }
        }
    }
}
