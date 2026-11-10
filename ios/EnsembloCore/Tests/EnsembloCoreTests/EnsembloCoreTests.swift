import Foundation
import Testing

@testable import EnsembloCore

private func idea(_ id: String, _ votes: [String: Bool], at second: Double) -> Idea {
    Idea(id: id, text: "Idée \(id)", createdBy: "u1", createdAt: Date(timeIntervalSince1970: second), votes: votes)
}

@Suite("Le décompte")
struct TallyTests {
    @Test("SplitMix64 donne les mêmes nombres que le site (Rust)")
    func sameNumbersAsTheWebsite() {
        #expect(Tally.mix(0) == 16_294_208_416_658_607_535)
        #expect(Tally.mix(42) == 13_679_457_532_755_275_413)
        #expect(Tally.mix(2_718_281_828) == 2_284_917_985_871_765_131)
    }

    @Test("L'idée qui a le plus de oui gagne")
    func theMostLikedWins() {
        let ideas = [
            idea("a", ["u1": true, "u2": false], at: 1),
            idea("b", ["u1": true, "u2": true, "u3": true], at: 2),
            idea("c", ["u1": false], at: 3),
        ]
        let tally = Tally(ideas: ideas, seed: 7)
        #expect(tally.winner?.id == "b")
        #expect(!tally.isTie)
        #expect(tally.ranking.map(\.idea.id) == ["b", "a", "c"])
        #expect(tally.ranking.first?.yes == 3)
        #expect(tally.ranking.last?.no == 1)
    }

    @Test("Une égalité se tire au sort parmi les ex æquo seulement", arguments: 0..<64)
    func aTieIsDrawnAmongTheLeaders(seed: UInt64) {
        let ideas = [
            idea("a", ["u1": true, "u2": true], at: 1),
            idea("b", ["u1": false], at: 2),
            idea("c", ["u2": true, "u3": true], at: 3),
        ]
        let tally = Tally(ideas: ideas, seed: seed)
        #expect(tally.isTie)
        #expect(tally.leaders.map(\.id) == ["a", "c"])
        #expect(["a", "c"].contains(tally.winner?.id ?? "-"))
        // Le même tirage sur chaque appareil.
        #expect(Tally(ideas: ideas, seed: seed).winner == tally.winner)
    }

    @Test("Chaque ex æquo peut gagner")
    func everyLeaderCanWin() {
        let leaders = ["a", "b", "c"]
        let winners = Set((0..<200).compactMap { Tally.draw(among: leaders, seed: $0) })
        #expect(winners == Set(leaders))
    }

    @Test("Sans idée ou sans oui, pas de gagnant")
    func noWinnerWithoutYes() {
        #expect(Tally(ideas: [], seed: 1).winner == nil)
        #expect(Tally(ideas: [idea("a", ["u1": false], at: 1)], seed: 1).winner == nil)
    }
}

@Suite("La session")
struct SessionTests {
    let session = Session(
        id: "SOIREE",
        createdAt: Date(timeIntervalSince1970: 1_000),
        creator: Participant(id: "u1", name: "Alice"),
        participants: [Participant(id: "u1", name: "Alice"), Participant(id: "u2", name: "Bastien")],
        ideas: [idea("a", ["u1": true], at: 1), idea("b", [:], at: 2)],
        seed: 9
    )

    @Test func pendingIdeasAndProgress() {
        #expect(session.pending(for: "u1").map(\.id) == ["b"])
        #expect(session.pending(for: "u2").map(\.id) == ["a", "b"])
        #expect(!session.hasFinished("u1"))
        var done = session
        done.ideas[1].votes["u1"] = false
        #expect(done.hasFinished("u1"))
        #expect(done.finished.map(\.id) == ["u1"])
    }

    @Test func expiresAfterADay() {
        #expect(!session.isExpired(at: Date(timeIntervalSince1970: 1_000 + 3_600)))
        #expect(session.isExpired(at: Date(timeIntervalSince1970: 1_000 + 86_401)))
    }
}

@Suite("Codes et liens")
struct CodeTests {
    @Test func randomCodesAreValid() {
        for _ in 0..<100 {
            let code = SessionCode.random()
            #expect(code.count == 6)
            #expect(SessionCode.normalize(code) == code)
        }
    }

    @Test(arguments: [
        ("soiree", "SOIREE"),
        (" k7q-2xm ", "K7Q2XM"),
        ("PIZZA9", "PIZZA9"),
        ("CINE42", "CINE42"),
        ("s0iree", "SOIREE"),
    ])
    func normalizesTypedCodes(input: String, code: String) {
        #expect(SessionCode.normalize(input) == code)
    }

    @Test(arguments: ["", "ABC", "TOOLONG1", "AB#DEF"])
    func refusesInvalidCodes(input: String) {
        #expect(SessionCode.normalize(input) == nil)
    }

    @Test func findsTheCodeInLinks() {
        #expect(SessionLink.code(in: "ensemblo://s/K7Q2XM") == "K7Q2XM")
        #expect(SessionLink.code(in: "https://exemple.fr/s/soiree") == "SOIREE")
        #expect(SessionLink.code(in: "https://exemple.fr/rejoindre?code=CINE42") == "CINE42")
        #expect(SessionLink.code(in: "PIZZA9") == "PIZZA9")
        #expect(SessionLink.code(in: "https://exemple.fr/autre") == nil)
    }

    @Test func makesLinks() {
        #expect(SessionLink.url(for: "SOIREE").absoluteString == "ensemblo://s/SOIREE")
        let web = SessionLink.url(for: "SOIREE", webBase: URL(string: "https://exemple.fr")!)
        #expect(web.absoluteString == "https://exemple.fr/s/SOIREE")
        #expect(SessionLink.code(in: web.absoluteString) == "SOIREE")
    }
}

@Suite("Firebase")
struct SnapshotTests {
    @Test func decodesASession() throws {
        let value: [String: Any] = [
            "createdAt": 1_731_276_000_000.0,
            "creator": ["id": "u1", "name": "Alice"],
            "participants": ["u2": "Bastien"],
            "seed": 2_718_281_828,
            "items": [
                "i2": ["text": "Ramen", "createdBy": "u2", "createdAt": 2_000, "votes": ["u1": true, "u2": false]],
                "i1": ["text": "Ciné", "createdBy": "u1", "createdAt": 1_000],
                "broken": ["votes": ["u1": true]],
            ],
        ]
        let session = try #require(Snapshot.session(code: "SOIREE", value: value))
        #expect(session.id == "SOIREE")
        #expect(session.creator.name == "Alice")
        #expect(Set(session.participants.map(\.id)) == ["u1", "u2"])
        #expect(session.ideas.map(\.text) == ["Ciné", "Ramen"])
        #expect(session.ideas[1].votes == ["u1": true, "u2": false])
        #expect(session.seed == 2_718_281_828)
        #expect(Snapshot.milliseconds(session.createdAt) == 1_731_276_000_000)
    }

    @Test func ignoresMissingSessions() {
        #expect(Snapshot.session(code: "SOIREE", value: nil) == nil)
        #expect(Snapshot.session(code: "SOIREE", value: ["items": [:]] as [String: Any]) == nil)
    }

    @Test func encodesASession() {
        let session = Session(
            id: "SOIREE",
            createdAt: Date(timeIntervalSince1970: 1_731_276_000),
            creator: Participant(id: "u1", name: "Alice"),
            participants: [Participant(id: "u1", name: "Alice")],
            seed: .max
        )
        let value = Snapshot.value(of: session)
        #expect(value["createdAt"] as? Int64 == 1_731_276_000_000)
        #expect((value["seed"] as? Int64).map { $0 < 1 << 53 } == true)
        // Ce qui est écrit se relit.
        let back = Snapshot.session(code: "SOIREE", value: value)
        #expect(back?.creator == session.creator)
    }
}
