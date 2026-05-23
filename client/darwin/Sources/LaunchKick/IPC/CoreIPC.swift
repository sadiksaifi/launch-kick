import Foundation

enum CoreIPCError: Error, Equatable {
    case invalidUTF8Line
    case invalidServerMessage(String)
    case clientEncodingFailed
}

final class CoreIPC {
    private var lineBuffer = NDJSONLineBuffer()
    private let contract: IPCContract
    private let input: FileHandle
    private let output: FileHandle
    private let callbackQueue: DispatchQueue

    var onResults: ((String, [LauncherResult]) -> Void)?
    var onActionResult: ((String, String, Bool, String?) -> Void)?
    var onError: ((CoreIPCError) -> Void)?

    init(
        input: FileHandle = .standardInput,
        output: FileHandle = .standardOutput,
        callbackQueue: DispatchQueue = .main,
        contract: IPCContract = IPCContract()
    ) {
        self.input = input
        self.output = output
        self.callbackQueue = callbackQueue
        self.contract = contract
    }

    deinit {
        stopListening()
    }

    func startListening() {
        input.readabilityHandler = { [weak self] handle in
            let data = handle.availableData
            if data.isEmpty {
                self?.stopListening()
                return
            }

            self?.callbackQueue.async {
                self?.handle(data)
            }
        }
    }

    func stopListening() {
        input.readabilityHandler = nil
    }

    func sendQuery(_ query: String) {
        sendToCore(.query(query))
    }

    func sendExecute(resultID: String, actionID: String) {
        sendToCore(.execute(resultID: resultID, actionID: actionID))
    }

    private func handle(_ data: Data) {
        for record in lineBuffer.append(data) {
            switch record {
            case let .line(line):
                handleLine(line)
            case .invalidUTF8:
                onError?(.invalidUTF8Line)
            }
        }
    }

    private func handleLine(_ line: String) {
        do {
            let message = try contract.decodeServerLine(line)
            switch message {
            case let .results(query, results):
                onResults?(query, results)
            case let .actionResult(resultID, actionID, ok, error):
                onActionResult?(resultID, actionID, ok, error)
            }
        } catch {
            onError?(.invalidServerMessage(line))
        }
    }

    private func sendToCore(_ message: ClientMessage) {
        guard
            let line = try? contract.encodeClientLine(message),
            let data = line.data(using: .utf8)
        else {
            onError?(.clientEncodingFailed)
            return
        }

        output.write(data)
        fflush(stdout)
    }
}
