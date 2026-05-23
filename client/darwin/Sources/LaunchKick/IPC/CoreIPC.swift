import Foundation

final class CoreIPC {
    private var stdinBuffer = ""
    private let contract = IPCContract()

    var onResults: ((String, [LauncherResult]) -> Void)?
    var onActionResult: ((String, String, Bool, String?) -> Void)?

    func startListening() {
        FileHandle.standardInput.readabilityHandler = { [weak self] handle in
            let data = handle.availableData
            if data.isEmpty { return }

            DispatchQueue.main.async {
                self?.handle(data)
            }
        }
    }

    func sendQuery(_ query: String) {
        sendToCore(QueryRequest(query: query))
    }

    func sendExecute(resultID: String, actionID: String) {
        sendToCore(ExecuteRequest(resultID: resultID, actionID: actionID))
    }

    private func handle(_ data: Data) {
        stdinBuffer += String(data: data, encoding: .utf8) ?? ""

        while let newline = stdinBuffer.firstIndex(of: "\n") {
            let line = String(stdinBuffer[..<newline])
            stdinBuffer.removeSubrange(...newline)
            handleLine(line)
        }
    }

    private func handleLine(_ line: String) {
        guard let message = try? contract.decodeServerLine(line) else { return }

        switch message {
        case .results(let query, let results):
            onResults?(query, results)
        case .actionResult(let resultID, let actionID, let ok, let error):
            onActionResult?(resultID, actionID, ok, error)
        }
    }

    private func sendToCore<Message: Encodable>(_ message: Message) {
        guard
            let line = try? contract.encodeClientLine(message),
            let data = line.data(using: .utf8)
        else { return }

        FileHandle.standardOutput.write(data)
        fflush(stdout)
    }
}
