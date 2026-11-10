import SwiftUI

/// Les couleurs et dégradés d'Ensemblo, les mêmes que sur le site.
enum Theme {
    static let night = Color(red: 13 / 255, green: 10 / 255, blue: 31 / 255)
    static let night2 = Color(red: 31 / 255, green: 23 / 255, blue: 71 / 255)
    static let coral = Color(red: 1, green: 107 / 255, blue: 107 / 255)
    static let violet = Color(red: 166 / 255, green: 107 / 255, blue: 1)
    static let mint = Color(red: 62 / 255, green: 224 / 255, blue: 208 / 255)
    static let yes = Color(red: 62 / 255, green: 224 / 255, blue: 160 / 255)
    static let no = Color(red: 1, green: 107 / 255, blue: 157 / 255)
    static let gold = Color(red: 1, green: 200 / 255, blue: 61 / 255)

    static var gradient: LinearGradient {
        LinearGradient(colors: [coral, violet, mint], startPoint: .topLeading, endPoint: .bottomTrailing)
    }

    static var background: some View {
        ZStack {
            night
            RadialGradient(colors: [violet.opacity(0.35), .clear], center: .topTrailing, startRadius: 0, endRadius: 520)
            RadialGradient(colors: [coral.opacity(0.18), .clear], center: .bottomLeading, startRadius: 0, endRadius: 480)
        }
    }

    /// La couleur d'une idée, stable d'un appareil à l'autre (tirée de son identifiant).
    static func hue(for id: String) -> Double {
        let hash = id.unicodeScalars.reduce(UInt32(2_166_136_261)) { ($0 ^ $1.value) &* 16_777_619 }
        return Double(hash % 360) / 360
    }

    static func card(for id: String) -> LinearGradient {
        let h = hue(for: id)
        return LinearGradient(
            colors: [
                Color(hue: h, saturation: 0.62, brightness: 0.98),
                Color(hue: (h + 0.1).truncatingRemainder(dividingBy: 1), saturation: 0.72, brightness: 0.62),
            ],
            startPoint: .topLeading,
            endPoint: .bottomTrailing
        )
    }

    /// Un emoji pour une idée, d'après ses mots (🎬 pour un film, 🍕 pour une pizza…).
    static func emoji(for text: String) -> String {
        let t = text.lowercased()
        let table: [([String], String)] = [
            (["ciné", "cine", "film", "série", "serie", "netflix"], "🎬"),
            (["pizza"], "🍕"),
            (["ramen", "sushi", "japonais", "resto", "restaurant", "dîner", "diner", "manger"], "🍜"),
            (["bar", "bière", "biere", "verre", "cocktail"], "🍻"),
            (["jeu", "jeux", "société"], "🎲"),
            (["karaoké", "karaoke", "chanter", "concert"], "🎤"),
            (["balade", "marche", "parc", "quais", "promenade"], "🌉"),
            (["bowling"], "🎳"),
            (["expo", "musée", "musee"], "🎨"),
            (["sport", "foot", "escalade", "piscine"], "🏃"),
            (["danse", "soirée", "soiree", "club"], "🪩"),
        ]
        return table.first { words, _ in words.contains { t.contains($0) } }?.1 ?? "✨"
    }
}

/// Un fond de verre (Liquid Glass) aux coins arrondis.
struct GlassCard: ViewModifier {
    var radius: CGFloat = 24

    func body(content: Content) -> some View {
        content
            .padding(18)
            .glassEffect(.regular, in: .rect(cornerRadius: radius))
    }
}

extension View {
    func glassCard(radius: CGFloat = 24) -> some View {
        modifier(GlassCard(radius: radius))
    }
}
