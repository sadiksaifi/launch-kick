import Foundation

enum CoreIPCStreamEvent: Equatable {
    case line(String)
    case invalidUTF8Line
    case closed
}

enum CoreIPCStreamError: Error, Equatable {
    case lineEncodingFailed
}

final class CoreIPCStream {
    private var lineBuffer = NDJSONLineBuffer()
    private let input: FileHandle
    private let output: FileHandle
    private let callbackQueue: DispatchQueue

    init(
        input: FileHandle = .standardInput,
        output: FileHandle = .standardOutput,
        callbackQueue: DispatchQueue = .main
    ) {
        self.input = input
        self.output = output
        self.callbackQueue = callbackQueue
    }

    deinit {
        stop()
    }

    func start(onEvent: @escaping (CoreIPCStreamEvent) -> Void) {
        input.readabilityHandler = { [weak self] handle in
            let data = handle.availableData
            guard !data.isEmpty else {
                self?.stop()
                self?.callbackQueue.async {
                    onEvent(.closed)
                }
                return
            }

            self?.callbackQueue.async {
                self?.handle(data, onEvent: onEvent)
            }
        }
    }

    func stop() {
        input.readabilityHandler = nil
    }

    func writeLine(_ line: String) throws {
        guard let data = line.data(using: .utf8) else {
            throw CoreIPCStreamError.lineEncodingFailed
        }

        output.write(data)
    }

    private func handle(_ data: Data, onEvent: (CoreIPCStreamEvent) -> Void) {
        for record in lineBuffer.append(data) {
            switch record {
            case let .line(line):
                onEvent(.line(line))
            case .invalidUTF8:
                onEvent(.invalidUTF8Line)
            }
        }
    }
}
