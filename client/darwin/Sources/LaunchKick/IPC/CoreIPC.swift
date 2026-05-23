import Foundation

struct LauncherApplication: Decodable {
    let name: String
    let path: String
}

private struct AppListRequest: Encodable {
    let type = "app::list"
}

private struct AppLaunchRequest: Encodable {
    let type = "app::launch"
    let path: String
}

private struct AppListResponse: Decodable {
    let type: String
    let apps: [LauncherApplication]
}

final class CoreIPC {
    private var stdinBuffer = ""
    private let decoder = JSONDecoder()
    private let encoder = JSONEncoder()
    var onAppList: (([LauncherApplication]) -> Void)?

    func startListening() {
        FileHandle.standardInput.readabilityHandler = { [weak self] handle in
            let data = handle.availableData
            if data.isEmpty { return }

            DispatchQueue.main.async {
                self?.handle(data)
            }
        }
    }

    func sendAppListRequest() {
        sendToCore(AppListRequest())
    }

    func sendAppLaunch(path: String) {
        sendToCore(AppLaunchRequest(path: path))
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
            let appList = try? decoder.decode(AppListResponse.self, from: data),
            appList.type == "app::list"
        else { return }

        onAppList?(appList.apps)
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
