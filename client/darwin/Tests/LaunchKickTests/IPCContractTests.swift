@testable import LaunchKick
import XCTest

final class IPCContractTests: XCTestCase {
    private let contract = IPCContract()

    func testManifestDrivesClientFixtureConformance() throws {
        for fixtureCase in try fixtureCatalog().cases.filter({ $0.direction == "client_to_core" }) {
            let message = try clientMessage(fromFixture: fixtureCase.file)
            let line = try contract.encodeClientLine(message)

            try XCTAssertJSONLine(line, equalsFixture: fixtureCase.file)
        }
    }

    func testManifestDrivesServerFixtureConformance() throws {
        for fixtureCase in try fixtureCatalog().cases.filter({ $0.direction == "core_to_client" }) {
            let message = try contract.decodeServerLine(fixture(fixtureCase.file))
            let normalized = try normalizedServerJSON(message)

            try XCTAssertJSON(normalized, equalsFixture: fixtureCase.file)
        }
    }

    func testRejectsUnknownServerMessage() throws {
        XCTAssertThrowsError(try contract.decodeServerLine("{\"type\":\"unknown\"}"))
    }

    func testManifestListsExistingFixtureFiles() throws {
        for fixtureCase in try fixtureCatalog().cases {
            XCTAssertTrue(
                FileManager.default.fileExists(atPath: fixtureURL(fixtureCase.file).path),
                "missing fixture file \(fixtureCase.file)"
            )
        }
    }

    func testEveryJSONFixtureFileIsListedInManifest() throws {
        let listed = try Set(fixtureCatalog().cases.map(\.file))
        let fixtureFiles = try FileManager.default.contentsOfDirectory(
            at: repoRoot().appendingPathComponent("ipc/fixtures"),
            includingPropertiesForKeys: nil
        )

        for fileURL in fixtureFiles {
            let fileName = fileURL.lastPathComponent
            guard fileName != "manifest.json", fileName.hasSuffix(".json") else { continue }

            XCTAssertTrue(listed.contains(fileName), "unlisted fixture file \(fileName)")
        }
    }

    func testManifestCaseMetadataMatchesFixtureTypes() throws {
        for fixtureCase in try fixtureCatalog().cases {
            let data = try Data(contentsOf: fixtureURL(fixtureCase.file))
            let json = try JSONSerialization.jsonObject(with: data) as? NSDictionary

            XCTAssertEqual(json?["type"] as? String, fixtureCase.type)
            XCTAssertTrue(["client_to_core", "core_to_client"].contains(fixtureCase.direction))
            XCTAssertFalse(fixtureCase.name.isEmpty)
        }
    }

    private func clientMessage(fromFixture fixtureName: String) throws -> ClientMessage {
        let data = try Data(contentsOf: fixtureURL(fixtureName))
        let json = try JSONSerialization.jsonObject(with: data) as? NSDictionary
        let type = try XCTUnwrap(json?["type"] as? String)

        switch type {
        case "launcher::query":
            return try .query(XCTUnwrap(json?["query"] as? String))
        case "launcher::execute":
            return try .execute(
                resultID: XCTUnwrap(json?["result_id"] as? String),
                actionID: XCTUnwrap(json?["action_id"] as? String)
            )
        default:
            XCTFail("unsupported client fixture type \(type)")
            return .query("")
        }
    }

    private func normalizedServerJSON(_ message: ServerMessage) throws -> String {
        let payload: [String: Any]
        switch message {
        case let .results(query, results):
            payload = [
                "type": "launcher::results",
                "query": query,
                "results": results.map(resultJSON),
            ]
        case let .actionResult(resultID, actionID, ok, error):
            var result: [String: Any] = [
                "type": "launcher::action::result",
                "result_id": resultID,
                "action_id": actionID,
                "ok": ok,
            ]
            if let error {
                result["error"] = error
            }
            payload = result
        }

        let data = try JSONSerialization.data(withJSONObject: payload, options: [])
        return String(data: data, encoding: .utf8) ?? "{}"
    }

    private func resultJSON(_ result: LauncherResult) -> [String: Any] {
        var json: [String: Any] = [
            "id": result.id,
            "title": result.title,
            "source": result.source,
            "actions": result.actions.map { ["id": $0.id, "title": $0.title] },
        ]
        if let subtitle = result.subtitle {
            json["subtitle"] = subtitle
        }
        if let icon = result.icon {
            json["icon"] = ["kind": icon.kind, "value": icon.value]
        }
        return json
    }

    private func XCTAssertJSONLine(_ line: String, equalsFixture fixtureName: String, file: StaticString = #filePath, line sourceLine: UInt = #line) throws {
        XCTAssertTrue(line.hasSuffix("\n"), file: file, line: sourceLine)
        try XCTAssertJSON(String(line.dropLast()), equalsFixture: fixtureName, file: file, line: sourceLine)
    }

    private func XCTAssertJSON(_ json: String, equalsFixture fixtureName: String, file: StaticString = #filePath, line sourceLine: UInt = #line) throws {
        let actual = try JSONSerialization.jsonObject(with: Data(json.utf8)) as? NSDictionary
        let expected = try JSONSerialization.jsonObject(with: Data(fixture(fixtureName).utf8)) as? NSDictionary
        XCTAssertEqual(actual, expected, file: file, line: sourceLine)
    }

    private func fixture(_ name: String) throws -> String {
        try String(contentsOf: fixtureURL(name), encoding: .utf8)
    }

    private func fixtureURL(_ name: String) -> URL {
        repoRoot().appendingPathComponent("ipc/fixtures/\(name)")
    }

    private func fixtureCatalog() throws -> FixtureCatalog {
        let data = try Data(contentsOf: fixtureURL("manifest.json"))
        return try JSONDecoder().decode(FixtureCatalog.self, from: data)
    }

    private func repoRoot() -> URL {
        var url = URL(fileURLWithPath: #filePath)
        while url.lastPathComponent != "launch-kick", url.path != url.deletingLastPathComponent().path {
            url.deleteLastPathComponent()
        }
        return url
    }
}

private struct FixtureCatalog: Decodable {
    let cases: [FixtureCase]
}

private struct FixtureCase: Decodable {
    let name: String
    let direction: String
    let type: String
    let file: String
}
