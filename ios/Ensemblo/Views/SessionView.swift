import EnsembloCore
import SwiftUI

/// Une session ouverte : les idées, le vote et le résultat, avec le code et le QR‑code
/// à partager.
struct SessionView: View {
    @Environment(SessionStore.self) private var store
    @State private var tab: Tab = .ideas
    @State private var sharing = false

    enum Tab: String, CaseIterable, Identifiable {
        case ideas = "Idées"
        case vote = "Voter"
        case result = "Résultat"
        var id: Self { self }
    }

    var body: some View {
        VStack(spacing: 16) {
            topBar
            Picker("Étape", selection: $tab) {
                ForEach(Tab.allCases) { Text($0.rawValue).tag($0) }
            }
            .pickerStyle(.segmented)
            .padding(.horizontal, 20)

            Group {
                if let session = store.session {
                    switch tab {
                    case .ideas: IdeasView(session: session) { tab = .vote }
                    case .vote: VoteView(session: session) { tab = .result }
                    case .result: ResultView(session: session)
                    }
                } else {
                    ProgressView("Connexion à la session…")
                        .frame(maxHeight: .infinity)
                }
            }
            .frame(maxWidth: 560)
            .animation(.smooth, value: tab)
        }
        .sheet(isPresented: $sharing) { ShareSheet() }
    }

    private var topBar: some View {
        HStack(spacing: 12) {
            Button {
                store.leave()
            } label: {
                Image(systemName: "chevron.left")
                    .font(.headline)
                    .frame(width: 44, height: 44)
            }
            .buttonStyle(.glass)
            .accessibilityLabel("Quitter la session")

            VStack(alignment: .leading, spacing: 2) {
                Text("Session").font(.caption).foregroundStyle(.secondary)
                Text(store.code ?? "")
                    .font(.system(.title2, design: .monospaced).weight(.bold))
                    .foregroundStyle(Theme.mint)
                    .contentTransition(.numericText())
            }
            Spacer()
            if let session = store.session {
                Label("\(session.participants.count)", systemImage: "person.2.fill")
                    .font(.subheadline.weight(.semibold))
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)
                    .glassEffect(.regular, in: .capsule)
                    .accessibilityLabel("\(session.participants.count) participants")
            }
            Button {
                sharing = true
            } label: {
                Image(systemName: "qrcode")
                    .font(.headline)
                    .frame(width: 44, height: 44)
            }
            .buttonStyle(.glassProminent)
            .accessibilityLabel("Inviter : QR‑code et lien")
        }
        .padding(.horizontal, 20)
        .padding(.top, 8)
    }
}

/// Le QR‑code de la session et le lien à partager.
struct ShareSheet: View {
    @Environment(SessionStore.self) private var store
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(spacing: 22) {
            Text("Invitez la bande").font(.title2.bold())
            if let link = store.link, let code = store.code {
                QRCodeView(text: link.absoluteString)
                    .frame(width: 230, height: 230)
                    .padding(14)
                    .background(.white, in: .rect(cornerRadius: 24))
                    .shadow(color: Theme.violet.opacity(0.6), radius: 30, y: 12)
                Text(code)
                    .font(.system(size: 40, weight: .heavy, design: .monospaced))
                    .kerning(6)
                    .foregroundStyle(Theme.gradient)
                Text("Scannez le QR‑code avec l'appareil photo, ou tapez le code dans Ensemblo.")
                    .multilineTextAlignment(.center)
                    .foregroundStyle(.secondary)
                ShareLink(
                    item: link,
                    subject: Text("On fait quoi ce soir ?"),
                    message: Text("Rejoins la session \(code) sur Ensemblo pour voter !")
                ) {
                    Label("Partager le lien", systemImage: "square.and.arrow.up")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.glassProminent)
                .controlSize(.large)
            }
            Button("Fermer") { dismiss() }
        }
        .padding(28)
        .presentationDetents([.large])
        .presentationBackground(Theme.night)
    }
}
