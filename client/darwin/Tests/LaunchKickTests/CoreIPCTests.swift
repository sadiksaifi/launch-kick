import Foundation
@testable import LaunchKick
import XCTest

final class CoreIPCTests: XCTestCase {
    private var listeningIPCs: [CoreIPC] = []

    override func tearDown() {
        for ipc in listeningIPCs {
            ipc.stopListening()
        }
        listeningIPCs.removeAll()
        super.tearDown()
    }

    func testSendQueryWritesClientQueryLine() throws {
        let output = Pipe()
        let ipc = CoreIPC(input: Pipe().fileHandleForReading, output: output.fileHandleForWriting)

        ipc.sendQuery("")

        let line = try XCTUnwrap(String(data: output.fileHandleForReading.availableData, encoding: .utf8))
        try XCTAssertJSONLine(line, equalsFixture: "client-query-empty.json")
    }

    func testSendExecuteWritesClientExecuteLine() throws {
        let output = Pipe()
        let ipc = CoreIPC(input: Pipe().fileHandleForReading, output: output.fileHandleForWriting)

        ipc.sendExecute(resultID: "application:/Applications/Safari.app", actionID: "open")

        let line = try XCTUnwrap(String(data: output.fileHandleForReading.availableData, encoding: .utf8))
        try XCTAssertJSONLine(line, equalsFixture: "client-execute-result.json")
    }

    func testIncomingResultsInvokesCallback() throws {
        let input = Pipe()
        let ipc = listeningIPC(input: input)
        let received = expectation(description: "received results")

        ipc.onResults = { query, results in
            XCTAssertEqual(query, "")
            XCTAssertEqual(results, [self.safariResult()])
            received.fulfill()
        }
        ipc.startListening()

        try input.fileHandleForWriting.write(Data((fixtureLine("server-results.json") + "\n").utf8))

        wait(for: [received], timeout: 2)
    }

    func testIncomingActionResultInvokesCallback() throws {
        let input = Pipe()
        let ipc = listeningIPC(input: input)
        let received = expectation(description: "received action result")

        ipc.onActionResult = { resultID, actionID, ok, error in
            XCTAssertEqual(resultID, "application:/Applications/Safari.app")
            XCTAssertEqual(actionID, "open")
            XCTAssertTrue(ok)
            XCTAssertNil(error)
            received.fulfill()
        }
        ipc.startListening()

        try input.fileHandleForWriting.write(Data((fixtureLine("server-action-succeeded.json") + "\n").utf8))

        wait(for: [received], timeout: 2)
    }

    func testMalformedServerLineReportsErrorAndContinues() throws {
        let input = Pipe()
        let ipc = listeningIPC(input: input)
        let reportedError = expectation(description: "reported error")
        let receivedResults = expectation(description: "received results")

        ipc.onError = { error in
            XCTAssertEqual(error, .invalidServerMessage("not json"))
            reportedError.fulfill()
        }
        ipc.onResults = { _, results in
            XCTAssertEqual(results, [self.safariResult()])
            receivedResults.fulfill()
        }
        ipc.startListening()

        let validLine = try fixtureLine("server-results.json")
        input.fileHandleForWriting.write(Data(("not json\n" + validLine + "\n").utf8))

        wait(for: [reportedError, receivedResults], timeout: 2)
    }

    private func listeningIPC(input: Pipe) -> CoreIPC {
        let output = Pipe()
        let ipc = CoreIPC(input: input.fileHandleForReading, output: output.fileHandleForWriting)
        listeningIPCs.append(ipc)
        return ipc
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
        try String(contentsOf: repoRoot().appendingPathComponent("ipc/fixtures/\(name)"), encoding: .utf8)
    }

    private func fixtureLine(_ name: String) throws -> String {
        try fixture(name).trimmingCharacters(in: .newlines)
    }

    private func repoRoot() -> URL {
        var url = URL(fileURLWithPath: #filePath)
        while url.lastPathComponent != "launch-kick", url.path != url.deletingLastPathComponent().path {
            url.deleteLastPathComponent()
        }
        return url
    }
}
