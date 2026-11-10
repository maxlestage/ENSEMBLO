// swift-tools-version: 6.4
// Le cœur d'Ensemblo, sans interface ni réseau : les modèles d'une session, le
// décompte des votes, le tirage au sort des égalités, les codes de session et les liens.
// Il se teste partout où Swift tourne (`swift test`), macOS comme Linux.

import PackageDescription

let package = Package(
    name: "EnsembloCore",
    platforms: [.iOS(.v26), .macOS(.v26)],
    products: [
        .library(name: "EnsembloCore", targets: ["EnsembloCore"]),
    ],
    targets: [
        .target(name: "EnsembloCore"),
        .testTarget(name: "EnsembloCoreTests", dependencies: ["EnsembloCore"]),
    ],
    swiftLanguageModes: [.v6]
)
