import Foundation
@testable import LaunchKick
import XCTest

final class NDJSONLineBufferTests: XCTestCase {
    func testReturnsCompleteLineFromSingleChunk() {
        var buffer = NDJSONLineBuffer()

        let lines = buffer.append(Data("one\n".utf8))

        XCTAssertEqual(lines, [.line("one")])
    }

    func testReturnsMultipleLinesFromSingleChunk() {
        var buffer = NDJSONLineBuffer()

        let lines = buffer.append(Data("one\ntwo\n".utf8))

        XCTAssertEqual(lines, [.line("one"), .line("two")])
    }

    func testPreservesPartialLineAcrossChunks() {
        var buffer = NDJSONLineBuffer()

        XCTAssertEqual(buffer.append(Data("on".utf8)), [])
        XCTAssertEqual(buffer.append(Data("e\n".utf8)), [.line("one")])
    }

    func testPreservesSplitMultibyteUTF8AcrossChunks() throws {
        var buffer = NDJSONLineBuffer()
        let data = Data("Safári\n".utf8)
        let splitIndex = try XCTUnwrap(data.firstIndex(of: 0xC3))

        XCTAssertEqual(buffer.append(Data(data[..<splitIndex])), [])
        XCTAssertEqual(buffer.append(Data(data[splitIndex...])), [.line("Safári")])
    }

    func testKeepsTrailingPartialLineBuffered() {
        var buffer = NDJSONLineBuffer()

        XCTAssertEqual(buffer.append(Data("one\nt".utf8)), [.line("one")])
        XCTAssertEqual(buffer.append(Data("wo".utf8)), [])
        XCTAssertEqual(buffer.append(Data("\n".utf8)), [.line("two")])
    }

    func testReturnsInvalidUTF8LineAndContinues() {
        var buffer = NDJSONLineBuffer()
        let invalidLine = Data([0xFF])
        var data = invalidLine
        data.append(0x0A)
        data.append(Data("valid\n".utf8))

        XCTAssertEqual(buffer.append(data), [.invalidUTF8(invalidLine), .line("valid")])
    }
}
