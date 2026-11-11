import EnsembloCore
import SwiftUI

/// Le résultat : le classement en direct, et le gagnant. En cas d'égalité, une roulette
/// passe d'un ex æquo à l'autre et s'arrête sur le gagnant du tirage, le même sur chaque
/// appareil (graine de la session).
struct ResultView: View {
    let session: Session

    @State private var spotlight: String?
    @State private var revealed = false
    @State private var spinning = false
    @State private var shown = false

    private var tally: Tally { session.result }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                verdict
                ForEach(Array(tally.ranking.enumerated()), id: \.element.id) { rank, row in
                    RankRow(
                        row: row,
                        max: max(tally.maxYes, 1),
                        lit: spotlight == row.id,
                        winner: (revealed || !tally.isTie) && tally.winner?.id == row.id,
                        shown: shown
                    )
                    .animation(.spring(response: 0.6, dampingFraction: 0.75).delay(Double(rank) * 0.07), value: shown)
                }
                Text("\(session.finished.count) sur \(session.participants.count) ont fini de voter.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
            .padding(20)
        }
        .onAppear { shown = true }
        .onChange(of: tally.leaders.map(\.id)) { revealed = false }
        .sensoryFeedback(.success, trigger: revealed)
        .sensoryFeedback(.selection, trigger: spotlight)
    }

    @ViewBuilder private var verdict: some View {
        if tally.leaders.isEmpty {
            Label("Pas encore de « oui ». Votez d'abord !", systemImage: "hourglass")
                .font(.headline)
                .glassCard()
        } else if let winner = tally.winner, !tally.isTie || revealed {
            WinnerCard(idea: winner, drawn: tally.isTie)
                .transition(.scale(scale: 0.6).combined(with: .opacity))
        } else {
            VStack(alignment: .leading, spacing: 12) {
                Label("Égalité entre \(tally.leaders.count) idées", systemImage: "equal.circle.fill")
                    .font(.headline)
                    .foregroundStyle(Theme.gold)
                Text("Le tirage au sort est le même sur tous les téléphones.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                Button {
                    Task { await spin() }
                } label: {
                    Label("Tirer au sort", systemImage: "dice.fill")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.glassProminent)
                .controlSize(.large)
                .disabled(spinning)
            }
            .glassCard()
        }
    }

    /// La roulette : au moins trois tours, de plus en plus lents, jusqu'au gagnant.
    private func spin() async {
        let leaders = tally.leaders.map(\.id)
        guard let winner = tally.winner?.id, let landing = leaders.firstIndex(of: winner) else { return }
        spinning = true
        let hops = leaders.count * 3 + landing + 1
        for hop in 0..<hops {
            withAnimation(.snappy(duration: 0.15)) { spotlight = leaders[hop % leaders.count] }
            let t = Double(hop) / Double(hops)
            try? await Task.sleep(for: .milliseconds(60 + Int(260 * t * t)))
        }
        withAnimation(.spring(response: 0.5, dampingFraction: 0.6)) {
            spotlight = nil
            revealed = true
        }
        spinning = false
    }
}

struct RankRow: View {
    let row: Tally.Row
    let max: Int
    let lit: Bool
    let winner: Bool
    let shown: Bool

    var body: some View {
        HStack(spacing: 12) {
            Text(Theme.emoji(for: row.idea.text)).font(.title2)
            VStack(alignment: .leading, spacing: 6) {
                HStack {
                    Text(row.idea.text).font(.subheadline.weight(.semibold)).lineLimit(1)
                    Spacer()
                    Text("\(row.yes) oui").font(.subheadline.monospacedDigit().bold())
                        .contentTransition(.numericText(value: Double(row.yes)))
                }
                GeometryReader { proxy in
                    Capsule()
                        .fill(Theme.card(for: row.idea.id))
                        .frame(width: shown ? proxy.size.width * CGFloat(row.yes) / CGFloat(max) : 0)
                }
                .frame(height: 8)
            }
        }
        .padding(12)
        .glassEffect(.regular, in: .rect(cornerRadius: 16))
        .overlay(
            RoundedRectangle(cornerRadius: 16)
                .stroke(Theme.gold, lineWidth: lit || winner ? 2 : 0)
        )
        .scaleEffect(lit ? 1.04 : 1)
        .shadow(color: Theme.gold.opacity(lit ? 0.7 : 0), radius: 16)
        .accessibilityElement(children: .combine)
        .accessibilityAddTraits(winner ? .isSelected : [])
    }
}

/// La carte du gagnant, qui apparaît en grandissant et en tournant à plat.
struct WinnerCard: View {
    let idea: Idea
    let drawn: Bool
    @State private var turned = false

    var body: some View {
        VStack(spacing: 10) {
            Label(drawn ? "Tiré au sort" : "Ce soir, c'est…", systemImage: drawn ? "dice.fill" : "crown.fill")
                .font(.subheadline.weight(.bold))
                .foregroundStyle(Theme.gold)
            Text(Theme.emoji(for: idea.text)).font(.system(size: 72))
            Text(idea.text)
                .font(.system(.title, design: .rounded).weight(.heavy))
                .multilineTextAlignment(.center)
        }
        .foregroundStyle(.white)
        .frame(maxWidth: .infinity)
        .padding(.vertical, 28)
        .background(Theme.card(for: idea.id), in: .rect(cornerRadius: 28))
        .scaleEffect(turned ? 1 : 0.4)
        .rotationEffect(.degrees(turned ? 0 : -12))
        .opacity(turned ? 1 : 0)
        .shadow(color: Theme.gold.opacity(0.5), radius: 24, y: 10)
        .onAppear {
            withAnimation(.spring(response: 0.8, dampingFraction: 0.6)) { turned = true }
        }
    }
}
