import Foundation

/// Les codes de session : six caractères faciles à dicter. Ni 0 ni 1, qu'on confondrait
/// avec O et I : 34 caractères, soit 1,5 milliard de codes.
public enum SessionCode {
    public static let alphabet = Array("ABCDEFGHIJKLMNOPQRSTUVWXYZ23456789")
    public static let length = 6

    /// Un nouveau code au hasard.
    public static func random<G: RandomNumberGenerator>(using generator: inout G) -> String {
        String((0..<length).map { _ in alphabet.randomElement(using: &generator)! })
    }

    public static func random() -> String {
        var generator = SystemRandomNumberGenerator()
        return random(using: &generator)
    }

    /// Le code tapé, en majuscules et sans espaces ni tirets ; `nil` s'il n'est pas valide.
    /// Un 0 ou un 1 tapé par erreur devient O ou I.
    public static func normalize(_ input: String) -> String? {
        let code = String(
            input.uppercased()
                .filter { !$0.isWhitespace && $0 != "-" }
                .map { $0 == "0" ? "O" : $0 == "1" ? "I" : $0 }
        )
        guard code.count == length, code.allSatisfy(alphabet.contains) else { return nil }
        return code
    }
}

/// Les liens vers une session, à partager ou à mettre dans un QR‑code.
public enum SessionLink {
    public static let scheme = "ensemblo"

    /// Le lien d'une session : sur le web si une adresse est configurée (les amis sans
    /// l'app l'ouvrent dans leur navigateur), sinon `ensemblo://s/CODE`.
    public static func url(for code: String, webBase: URL? = nil) -> URL {
        if let webBase {
            return webBase.appending(path: "s").appending(path: code)
        }
        return URL(string: "\(scheme)://s/\(code)")!
    }

    /// Le code d'une session dans un lien (`ensemblo://s/CODE`, `https://…/s/CODE`,
    /// `…?code=CODE`) ou dans un texte qui n'est que le code.
    public static func code(in text: String) -> String? {
        let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
        if let code = SessionCode.normalize(trimmed) {
            return code
        }
        guard let url = URL(string: trimmed), url.scheme != nil else { return nil }
        if let query = URLComponents(url: url, resolvingAgainstBaseURL: false)?.queryItems,
           let value = query.first(where: { $0.name == "code" })?.value,
           let code = SessionCode.normalize(value) {
            return code
        }
        // `ensemblo://s/CODE` : « s » est l'hôte ; `https://exemple.fr/s/CODE` : un chemin.
        var parts = url.pathComponents.filter { $0 != "/" }
        if url.scheme == scheme, let host = url.host() {
            parts.insert(host, at: 0)
        }
        guard let s = parts.lastIndex(of: "s"), s + 1 < parts.count else { return nil }
        return SessionCode.normalize(parts[s + 1])
    }
}
