import XCTest
@testable import LaunchKick

final class IPCContractTests: XCTestCase {
    private let contract = IPCContract()

    func testEncodesAppListRequest() throws {
        let line = try contract.encodeClientLine(AppListRequest())

        try XCTAssertJSONLine(line, equalsFixture: "client-app-list.json")
    }

    func testEncodesAppLaunchRequest() throws {
        let line = try contract.encodeClientLine(AppLaunchRequest(path: "/Applications/Safari.app"))

        try XCTAssertJSONLine(line, equalsFixture: "client-app-launch.json")
    }

    func testDecodesAppListResponse() throws {
        let message = try contract.decodeServerLine(fixture("server-app-list.json"))

        XCTAssertEqual(
            message,
            .appList([
                LauncherApplication(name: "Safari", path: "/Applications/Safari.app")
            ])
        )
    }

    func testDecodesLaunchSuccessResponse() throws {
        let message = try contract.decodeServerLine(fixture("server-app-launch-succeeded.json"))

        XCTAssertEqual(
            message,
            .appLaunchResult(path: "/Applications/Safari.app", ok: true, error: nil)
        )
    }

    func testDecodesLaunchFailureResponse() throws {
        let message = try contract.decodeServerLine(fixture("server-app-launch-failed.json"))

        XCTAssertEqual(
            message,
            .appLaunchResult(path: "/Applications/Missing.app", ok: false, error: "launch failed")
        )
    }

    func testRejectsUnknownServerMessage() throws {
        XCTAssertThrowsError(try contract.decodeServerLine("{\"type\":\"unknown\"}"))
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
