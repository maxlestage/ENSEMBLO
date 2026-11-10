import EnsembloCore
import FirebaseCore
import SwiftUI

@main
struct EnsembloApp: App {
    @State private var store: SessionStore

    init() {
        // Sans `GoogleService-Info.plist` (voir ios/README.md), l'app tourne en local.
        let configured = Bundle.main.path(forResource: "GoogleService-Info", ofType: "plist") != nil
        if configured {
            FirebaseApp.configure()
        }
        let web = (Bundle.main.object(forInfoDictionaryKey: "EnsembloWebURL") as? String)
            .flatMap { $0.isEmpty ? nil : URL(string: $0) }
        _store = State(initialValue: SessionStore(
            backend: configured ? FirebaseBackend() : LocalBackend(),
            isLocal: !configured,
            webBase: web
        ))
    }

    var body: some Scene {
        WindowGroup {
            RootView()
                .environment(store)
                .preferredColorScheme(.dark)
                .tint(Theme.violet)
        }
    }
}

/// L'accueil, ou la session ouverte. Un lien `ensemblo://s/CODE` ouvre la session.
struct RootView: View {
    @Environment(SessionStore.self) private var store
    @AppStorage("ensemblo.name") private var name = ""

    var body: some View {
        ZStack {
            Theme.background.ignoresSafeArea()
            if store.code != nil {
                SessionView()
                    .transition(.asymmetric(insertion: .move(edge: .trailing), removal: .opacity))
            } else {
                HomeView()
                    .transition(.opacity)
            }
        }
        .animation(.smooth(duration: 0.45), value: store.code)
        .task { await store.start() }
        .onOpenURL { url in
            guard let code = SessionLink.code(in: url.absoluteString) else { return }
            Task { await store.join(code, name: name) }
        }
        .alert(
            "Oups",
            isPresented: Binding(get: { store.error != nil }, set: { if !$0 { store.error = nil } }),
            actions: { Button("OK", role: .cancel) {} },
            message: { Text(store.error ?? "") }
        )
    }
}
