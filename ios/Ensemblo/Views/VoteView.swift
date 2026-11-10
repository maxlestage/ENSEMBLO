import EnsembloCore
import SwiftUI

/// Le vote : une pile de cartes. À droite c'est oui, à gauche c'est non.
struct VoteView: View {
    let session: Session
    var onDone: () -> Void

    @Environment(SessionStore.self) private var store
    @State private var drag: CGSize = .zero
    @State private var flying: (id: String, yes: Bool)?
    @State private var votes = 0

    private let threshold: CGFloat = 110

    private var pending: [Idea] {
        guard let userId = store.userId else { return [] }
        return session.pending(for: userId)
    }

    var body: some View {
        VStack(spacing: 18) {
            progress
            ZStack {
                if pending.isEmpty {
                    finished
                } else {
                    ForEach(Array(pending.prefix(3).enumerated().reversed()), id: \.element.id) { index, idea in
                        card(idea, depth: index)
                    }
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .padding(.horizontal, 24)

            if let top = pending.first {
                HStack(spacing: 40) {
                    voteButton(yes: false, idea: top)
                    voteButton(yes: true, idea: top)
                }
                .padding(.bottom, 12)
            }
        }
        .sensoryFeedback(.impact(weight: .medium), trigger: votes)
    }

    private var progress: some View {
        HStack(spacing: -8) {
            ForEach(session.participants) { person in
                Text(String(person.name.prefix(1)))
                    .font(.footnote.bold())
                    .frame(width: 34, height: 34)
                    .background(Theme.night2, in: .circle)
                    .overlay(Circle().stroke(session.hasFinished(person.id) ? Theme.yes : .white.opacity(0.15), lineWidth: 2))
                    .accessibilityLabel("\(person.name)\(session.hasFinished(person.id) ? ", a fini" : "")")
            }
            Text("\(session.finished.count)/\(session.participants.count) ont fini")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .padding(.leading, 18)
        }
    }

    private func card(_ idea: Idea, depth: Int) -> some View {
        let isTop = depth == 0
        let offset = isTop ? drag : .zero
        let flew = flying?.id == idea.id
        return SwipeCard(idea: idea, author: author(of: idea), yes: isTop ? drag.width / threshold : 0)
            .scaleEffect(1 - CGFloat(depth) * 0.05)
            .offset(y: CGFloat(depth) * 14)
            .offset(x: flew ? (flying!.yes ? 600 : -600) : offset.width, y: flew ? -60 : offset.height * 0.3)
            .rotationEffect(.degrees(flew ? (flying!.yes ? 24 : -24) : Double(offset.width / 18)))
            .rotation3DEffect(.degrees(Double(offset.width / 10)), axis: (x: 0, y: 1, z: 0), perspective: 0.6)
            .allowsHitTesting(isTop)
            .gesture(
                DragGesture()
                    .onChanged { drag = $0.translation }
                    .onEnded { value in
                        let predicted = value.predictedEndTranslation.width
                        if abs(predicted) > threshold || abs(value.translation.width) > threshold {
                            cast(predicted > 0, on: idea)
                        } else {
                            withAnimation(.spring(response: 0.4, dampingFraction: 0.65)) { drag = .zero }
                        }
                    }
            )
            .accessibilityElement(children: .combine)
            .accessibilityActions {
                Button("Oui") { cast(true, on: idea) }
                Button("Non") { cast(false, on: idea) }
            }
    }

    private func voteButton(yes: Bool, idea: Idea) -> some View {
        Button {
            cast(yes, on: idea)
        } label: {
            Image(systemName: yes ? "heart.fill" : "xmark")
                .font(.title.bold())
                .foregroundStyle(yes ? Theme.yes : Theme.no)
                .frame(width: 72, height: 72)
        }
        .buttonStyle(.glass)
        .buttonBorderShape(.circle)
        .accessibilityLabel(yes ? "Oui" : "Non")
    }

    private var finished: some View {
        VStack(spacing: 16) {
            Image(systemName: "checkmark.seal.fill")
                .font(.system(size: 64))
                .foregroundStyle(Theme.gradient)
                .symbolEffect(.bounce, value: votes)
            Text(session.ideas.isEmpty ? "Ajoutez des idées pour voter." : "Vous avez tout voté !")
                .font(.title2.bold())
            Text("Le résultat se met à jour à chaque vote.")
                .foregroundStyle(.secondary)
            if !session.ideas.isEmpty {
                Button("Voir le résultat", action: onDone)
                    .buttonStyle(.glassProminent)
                    .controlSize(.large)
            }
        }
        .multilineTextAlignment(.center)
        .transition(.scale.combined(with: .opacity))
    }

    private func cast(_ yes: Bool, on idea: Idea) {
        votes += 1
        withAnimation(.easeIn(duration: 0.28)) {
            flying = (idea.id, yes)
        } completion: {
            store.vote(yes, on: idea)
            drag = .zero
            flying = nil
        }
    }

    private func author(of idea: Idea) -> String {
        session.participants.first { $0.id == idea.createdBy }?.name ?? "?"
    }
}

/// Une carte d'idée, avec les tampons OUI / NON qui apparaissent quand on la glisse.
struct SwipeCard: View {
    let idea: Idea
    let author: String
    /// De -1 (non) à 1 (oui), selon le glissement.
    var yes: CGFloat

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Spacer()
            Text(Theme.emoji(for: idea.text))
                .font(.system(size: 84))
                .shadow(color: .black.opacity(0.3), radius: 12, y: 10)
            Text(idea.text)
                .font(.system(.largeTitle, design: .rounded).weight(.heavy))
                .minimumScaleFactor(0.6)
                .lineLimit(3)
            Divider().overlay(.white.opacity(0.3))
            Text("Proposé par \(author)")
                .font(.subheadline)
                .opacity(0.85)
        }
        .foregroundStyle(.white)
        .padding(26)
        .frame(maxWidth: 380, maxHeight: 500)
        .background(Theme.card(for: idea.id), in: .rect(cornerRadius: 32))
        .overlay(alignment: .topLeading) {
            stamp("OUI", color: Theme.yes, angle: -14).opacity(Double(max(0, min(1, yes))))
        }
        .overlay(alignment: .topTrailing) {
            stamp("NON", color: Theme.no, angle: 14).opacity(Double(max(0, min(1, -yes))))
        }
        .shadow(color: .black.opacity(0.45), radius: 24, y: 16)
    }

    private func stamp(_ text: String, color: Color, angle: Double) -> some View {
        Text(text)
            .font(.system(size: 34, weight: .black, design: .rounded))
            .foregroundStyle(color)
            .padding(.horizontal, 14)
            .padding(.vertical, 6)
            .overlay(RoundedRectangle(cornerRadius: 12).stroke(color, lineWidth: 4))
            .rotationEffect(.degrees(angle))
            .padding(28)
    }
}
