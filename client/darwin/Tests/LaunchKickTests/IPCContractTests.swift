@testable import LaunchKick
import XCTest

final class IPCContractTests: XCTestCase {
    private let contract = IPCContract()

    func testEncodesQueryRequest() throws {
        let line = try contract.encodeClientLine(.query(""))

        try XCTAssertJSONLine(line, equalsFixture: "client-query-empty.json")
    }

    func testEncodesTextQueryRequest() throws {
        let line = try contract.encodeClientLine(.query("saf"))

        try XCTAssertJSONLine(line, equalsFixture: "client-query-safari.json")
    }

    func testEncodesExecuteRequest() throws {
        let line = try contract.encodeClientLine(.execute(
            resultID: "application:/Applications/Safari.app",
            actionID: "open"
        ))

        try XCTAssertJSONLine(line, equalsFixture: "client-execute-result.json")
    }

    func testDecodesResultsResponse() throws {
        let message = try contract.decodeServerLine(fixture("server-results.json"))

        XCTAssertEqual(
            message,
            .results(query: "", results: [safariResult()])
        )
    }

    func testDecodesActionSuccessResponse() throws {
        let message = try contract.decodeServerLine(fixture("server-action-succeeded.json"))

        XCTAssertEqual(
            message,
            .actionResult(
                resultID: "application:/Applications/Safari.app",
                actionID: "open",
                ok: true,
                error: nil
            )
        )
    }

    func testDecodesActionFailureResponse() throws {
        let message = try contract.decodeServerLine(fixture("server-action-failed.json"))

        XCTAssertEqual(
            message,
            .actionResult(
                resultID: "application:/Applications/Missing.app",
                actionID: "open",
                ok: false,
                error: "launch failed"
            )
        )
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

    private func safariResult() -> LauncherResult {
        LauncherResult(
            id: "application:/Applications/Safari.app",
            title: "Safari",
            subtitle: "/Applications/Safari.app",
            source: "applications",
            icon: IconDescriptor(kind: "file", value: "/Applications/Safari.app"),
            actions: [LauncherAction(id: "open", title: "Open")]
        )
    }

    private func XCTAssertJSONLine(_ line: String, equalsFixture fixtureName: String, file: StaticString = #filePath, line sourceLine: UInt = #line) throws {
        XCTAssertTrue(line.hasSuffix("\n"), file: file, line: sourceLine)
        let actual = try JSONSerialization.jsonObject(with: Data(line.utf8)) as? NSDictionary
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
