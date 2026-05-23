import Foundation
@testable import LaunchKick
import XCTest

final class NDJSONLineBufferTests: XCTestCase {
    func testReturnsCompleteLineFromSingleChunk() {
        var buffer = NDJSONLineBuffer()

        let lines = buffer.append(Data("one\n".utf8))

        XCTAssertEqual(lines, ["one"])
    }

    func testReturnsMultipleLinesFromSingleChunk() {
        var buffer = NDJSONLineBuffer()

        let lines = buffer.append(Data("one\ntwo\n".utf8))

        XCTAssertEqual(lines, ["one", "two"])
    }

    func testPreservesPartialLineAcrossChunks() {
        var buffer = NDJSONLineBuffer()

        XCTAssertEqual(buffer.append(Data("on".utf8)), [])
        XCTAssertEqual(buffer.append(Data("e\n".utf8)), ["one"])
    }

    func testPreservesSplitMultibyteUTF8AcrossChunks() throws {
        var buffer = NDJSONLineBuffer()
        let data = Data("Safári\n".utf8)
        let splitIndex = try XCTUnwrap(data.firstIndex(of: 0xC3))

        XCTAssertEqual(buffer.append(Data(data[..<splitIndex])), [])
        XCTAssertEqual(buffer.append(Data(data[splitIndex...])), ["Safári"])
    }

    func testKeepsTrailingPartialLineBuffered() {
        var buffer = NDJSONLineBuffer()

        XCTAssertEqual(buffer.append(Data("one\nt".utf8)), ["one"])
        XCTAssertEqual(buffer.append(Data("wo".utf8)), [])
        XCTAssertEqual(buffer.append(Data("\n".utf8)), ["two"])
    }
}
