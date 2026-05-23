import Foundation

final class CoreIPC {
    private var lineBuffer = NDJSONLineBuffer()
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
        for line in lineBuffer.append(data) {
            handleLine(line)
        }
    }

    private func handleLine(_ line: String) {
        guard let message = try? contract.decodeServerLine(line) else { return }

        switch message {
        case let .results(query, results):
            onResults?(query, results)
        case let .actionResult(resultID, actionID, ok, error):
            onActionResult?(resultID, actionID, ok, error)
        }
    }

    private func sendToCore(_ message: some Encodable) {
        guard
            let line = try? contract.encodeClientLine(message),
            let data = line.data(using: .utf8)
        else { return }

        FileHandle.standardOutput.write(data)
        fflush(stdout)
    }
}
