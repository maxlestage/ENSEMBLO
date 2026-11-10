import EnsembloCore
import SwiftUI

/// Les idées de la session : chacun ajoute les siennes, elles apparaissent chez tout le monde.
struct IdeasView: View {
    let session: Session
    var onVote: () -> Void

    @Environment(SessionStore.self) private var store
    @State private var text = ""
    @FocusState private var focused: Bool

    private let suggestions = ["🎬 Ciné", "🍕 Pizza & série", "🎲 Jeux de société", "🍻 Un verre", "🌉 Balade"]

    var body: some View {
        VStack(spacing: 14) {
            HStack(spacing: 10) {
                TextField("Une idée pour ce soir…", text: $text)
                    .focused($focused)
                    .submitLabel(.send)
                    .onSubmit(add)
                    .padding(14)
                    .glassEffect(.regular.interactive(), in: .rect(cornerRadius: 16))
                Button(action: add) {
                    Image(systemName: "plus")
                        .font(.title3.bold())
                        .frame(width: 48, height: 48)
                }
                .buttonStyle(.glassProminent)
                .disabled(text.trimmingCharacters(in: .whitespaces).isEmpty)
                .accessibilityLabel("Ajouter l'idée")
            }
            .padding(.horizontal, 20)

            ScrollView(.horizontal) {
                HStack {
                    ForEach(suggestions, id: \.self) { suggestion in
                        Button(suggestion) {
                            text = String(suggestion.drop(while: { !$0.isLetter }))
                            focused = true
                        }
                        .buttonStyle(.glass)
                        .font(.footnote)
                    }
                }
                .padding(.horizontal, 20)
            }
            .scrollIndicators(.hidden)

            if session.ideas.isEmpty {
                ContentUnavailableView(
                    "Pas encore d'idée",
                    systemImage: "lightbulb",
                    description: Text("Proposez la première : un film, un resto, une activité…")
                )
                .frame(maxHeight: .infinity)
            } else {
                List {
                    ForEach(session.ideas) { idea in
                        IdeaRow(idea: idea, author: author(of: idea))
                            .listRowBackground(Color.clear)
                            .listRowSeparator(.hidden)
                            .transition(.move(edge: .top).combined(with: .opacity))
                    }
                }
                .listStyle(.plain)
                .scrollContentBackground(.hidden)
                .animation(.bouncy, value: session.ideas.map(\.id))

                Button(action: onVote) {
                    Label("Passer au vote", systemImage: "hand.draw")
                        .font(.headline)
                        .frame(maxWidth: .infinity, minHeight: 32)
                }
                .buttonStyle(.glassProminent)
                .controlSize(.large)
                .padding(.horizontal, 20)
                .padding(.bottom, 8)
            }
        }
    }

    private func add() {
        store.add(idea: text)
        text = ""
    }

    private func author(of idea: Idea) -> String {
        session.participants.first { $0.id == idea.createdBy }?.name ?? "?"
    }
}

struct IdeaRow: View {
    let idea: Idea
    let author: String

    var body: some View {
        HStack(spacing: 14) {
            Text(Theme.emoji(for: idea.text))
                .font(.title)
                .frame(width: 52, height: 52)
                .background(Theme.card(for: idea.id), in: .rect(cornerRadius: 14))
            VStack(alignment: .leading, spacing: 2) {
                Text(idea.text).font(.headline)
                Text("Proposé par \(author)").font(.caption).foregroundStyle(.secondary)
            }
            Spacer(minLength: 0)
        }
        .padding(10)
        .glassEffect(.regular, in: .rect(cornerRadius: 18))
    }
}
