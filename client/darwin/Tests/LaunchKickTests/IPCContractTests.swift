import XCTest
@testable import LaunchKick

final class IPCContractTests: XCTestCase {
    private let contract = IPCContract()

    func testEncodesQueryRequest() throws {
        let line = try contract.encodeClientLine(QueryRequest(query: ""))

        try XCTAssertJSONLine(line, equalsFixture: "client-query-empty.json")
    }

    func testEncodesTextQueryRequest() throws {
        let line = try contract.encodeClientLine(QueryRequest(query: "saf"))

        try XCTAssertJSONLine(line, equalsFixture: "client-query-safari.json")
    }

    func testEncodesExecuteRequest() throws {
        let line = try contract.encodeClientLine(ExecuteRequest(
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
        let data = try Data(contentsOf: repoRoot().appendingPathComponent("ipc/fixtures/\(name)"))
        return String(decoding: data, as: UTF8.self)
    }

    private func repoRoot() -> URL {
        var url = URL(fileURLWithPath: #filePath)
        while url.lastPathComponent != "launch-kick" && url.path != url.deletingLastPathComponent().path {
            url.deleteLastPathComponent()
        }
        return url
    }
}
