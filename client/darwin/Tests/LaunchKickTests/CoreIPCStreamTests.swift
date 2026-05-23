import Foundation
@testable import LaunchKick
import XCTest

final class CoreIPCStreamTests: XCTestCase {
    private var streams: [CoreIPCStream] = []

    override func tearDown() {
        for stream in streams {
            stream.stop()
        }
        streams.removeAll()
        super.tearDown()
    }

    func testEmitsCompleteLinesFromInput() {
        let input = Pipe()
        let stream = listeningStream(input: input)
        let received = expectation(description: "received lines")
        var events: [CoreIPCStreamEvent] = []

        stream.start { event in
            events.append(event)
            if events.count == 2 {
                received.fulfill()
            }
        }

        input.fileHandleForWriting.write(Data("one\ntwo\n".utf8))

        wait(for: [received], timeout: 2)
        XCTAssertEqual(events, [.line("one"), .line("two")])
    }

    func testEmitsInvalidUTF8AndContinues() {
        let input = Pipe()
        let stream = listeningStream(input: input)
        let received = expectation(description: "received events")
        var events: [CoreIPCStreamEvent] = []

        stream.start { event in
            events.append(event)
            if events.count == 2 {
                received.fulfill()
            }
        }

        var data = Data([0xFF, 0x0A])
        data.append(Data("valid\n".utf8))
        input.fileHandleForWriting.write(data)

        wait(for: [received], timeout: 2)
        XCTAssertEqual(events, [.invalidUTF8Line, .line("valid")])
    }

    func testWriteLineWritesToOutput() throws {
        let output = Pipe()
        let stream = CoreIPCStream(input: Pipe().fileHandleForReading, output: output.fileHandleForWriting)

        try stream.writeLine("hello\n")

        let line = String(data: output.fileHandleForReading.availableData, encoding: .utf8)
        XCTAssertEqual(line, "hello\n")
    }

    private func listeningStream(input: Pipe) -> CoreIPCStream {
        let stream = CoreIPCStream(input: input.fileHandleForReading, output: Pipe().fileHandleForWriting)
        streams.append(stream)
        return stream
    }
}
