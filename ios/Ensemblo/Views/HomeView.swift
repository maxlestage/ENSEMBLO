import EnsembloCore
import SwiftUI

/// L'accueil : un prénom, puis créer une session ou en rejoindre une (code ou QR‑code).
struct HomeView: View {
    @Environment(SessionStore.self) private var store
    @AppStorage("ensemblo.name") private var name = ""
    @State private var code = ""
    @State private var scanning = false
    @State private var floating = false
    @FocusState private var focused: Field?

    private enum Field { case name, code }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 28) {
                header
                nameField
                createButton
                joinCard
                if store.isLocal {
                    Label(
                        "Mode local : Firebase n'est pas configuré, les sessions restent sur cet appareil.",
                        systemImage: "wifi.slash"
                    )
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                }
            }
            .padding(.horizontal, 20)
            .padding(.vertical, 32)
            .frame(maxWidth: 560)
            .frame(maxWidth: .infinity)
        }
        .scrollDismissesKeyboard(.interactively)
        .sheet(isPresented: $scanning) {
            ScannerSheet { payload in
                scanning = false
                Task { await store.join(payload, name: name) }
            }
        }
        .onAppear { floating = true }
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack(spacing: -18) {
                ForEach(Array(["🎬", "🍜", "🎲"].enumerated()), id: \.offset) { index, emoji in
                    Text(emoji)
                        .font(.system(size: 38))
                        .frame(width: 66, height: 86)
                        .background(Theme.card(for: emoji), in: .rect(cornerRadius: 16))
                        .rotationEffect(.degrees(Double(index - 1) * 10))
                        .rotationEffect(.degrees(floating ? 4 : -4))
                        .offset(y: floating ? -6 : 6)
                        .animation(
                            .easeInOut(duration: 2.4).repeatForever().delay(Double(index) * 0.3),
                            value: floating
                        )
                        .shadow(color: .black.opacity(0.4), radius: 12, y: 8)
                }
            }
            .padding(.bottom, 6)
            .accessibilityHidden(true)

            Text("On fait quoi\nce soir ?")
                .font(.system(size: 46, weight: .heavy, design: .rounded))
                .foregroundStyle(Theme.gradient)
            Text("Chacun propose, tout le monde swipe, Ensemblo tranche.")
                .font(.title3)
                .foregroundStyle(.secondary)
        }
    }

    private var nameField: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Votre prénom").font(.subheadline.weight(.semibold)).foregroundStyle(.secondary)
            TextField("Alice", text: $name)
                .textContentType(.givenName)
                .submitLabel(.done)
                .focused($focused, equals: .name)
                .font(.title3)
                .padding(14)
                .glassEffect(.regular.interactive(), in: .rect(cornerRadius: 16))
        }
    }

    private var createButton: some View {
        Button {
            focused = nil
            Task { await store.create(name: name) }
        } label: {
            Label("Créer une session", systemImage: "sparkles")
                .font(.headline)
                .frame(maxWidth: .infinity, minHeight: 34)
        }
        .buttonStyle(.glassProminent)
        .controlSize(.large)
        .disabled(store.isWorking)
        .sensoryFeedback(.success, trigger: store.code)
    }

    private var joinCard: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Rejoindre").font(.headline)
            HStack(spacing: 10) {
                TextField("Code à 6 caractères", text: $code)
                    .textInputAutocapitalization(.characters)
                    .autocorrectionDisabled()
                    .font(.system(.title3, design: .monospaced).weight(.semibold))
                    .focused($focused, equals: .code)
                    .submitLabel(.join)
                    .onSubmit(join)
                    .padding(12)
                    .background(.white.opacity(0.06), in: .rect(cornerRadius: 14))
                Button("Entrer", action: join)
                    .buttonStyle(.glass)
                    .disabled(SessionCode.normalize(code) == nil || store.isWorking)
            }
            Button {
                scanning = true
            } label: {
                Label("Scanner un QR‑code", systemImage: "qrcode.viewfinder")
                    .frame(maxWidth: .infinity)
            }
            .buttonStyle(.glass)
            .controlSize(.large)
        }
        .glassCard()
    }

    private func join() {
        focused = nil
        Task { await store.join(code, name: name) }
    }
}

#Preview {
    HomeView()
        .environment(SessionStore(backend: LocalBackend(), isLocal: true))
        .background(Theme.background)
        .preferredColorScheme(.dark)
}
