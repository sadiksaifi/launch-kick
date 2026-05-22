import Foundation

private struct ClientInputMessage: Encodable {
    let type = "input"
    let text: String
}

private struct ServerResultMessage: Decodable {
    let type: String
    let value: String
}

final class CoreIPC {
    private var stdinBuffer = ""
    private let decoder = JSONDecoder()
    private let encoder = JSONEncoder()
    var onResult: ((String) -> Void)?

    func startListening() {
        FileHandle.standardInput.readabilityHandler = { [weak self] handle in
            let data = handle.availableData
            if data.isEmpty { return }

            DispatchQueue.main.async {
                self?.handle(data)
            }
        }
    }

    func sendInput(_ text: String) {
        sendToCore(ClientInputMessage(text: text))
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
        guard
            let data = line.data(using: .utf8),
            let result = try? decoder.decode(ServerResultMessage.self, from: data),
            result.type == "result"
        else { return }

        onResult?(result.value)
    }

    private func sendToCore<Message: Encodable>(_ message: Message) {
        guard
            let data = try? encoder.encode(message),
            let line = String(data: data, encoding: .utf8)
        else { return }

        print(line)
        fflush(stdout)
    }
}
