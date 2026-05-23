import Foundation

struct LauncherApplication: Codable, Equatable {
    let name: String
    let path: String
}

enum ClientMessageType {
    static let appList = "app::list"
    static let appLaunch = "app::launch"
}

enum ServerMessageType {
    static let appList = "app::list"
    static let appLaunchResult = "app::launch::result"
}

struct AppListRequest: Encodable {
    let type = ClientMessageType.appList
}

struct AppLaunchRequest: Encodable {
    let type = ClientMessageType.appLaunch
    let path: String
}

enum ServerMessage: Equatable {
    case appList([LauncherApplication])
    case appLaunchResult(path: String, ok: Bool, error: String?)
}

struct IPCContract {
    private let decoder = JSONDecoder()
    private let encoder = JSONEncoder()

    func encodeClientLine<Message: Encodable>(_ message: Message) throws -> String {
        let data = try encoder.encode(message)
        guard let json = String(data: data, encoding: .utf8) else {
            throw IPCContractError.invalidUTF8
        }
        return json + "\n"
    }

    func decodeServerLine(_ line: String) throws -> ServerMessage {
        guard let data = line.data(using: .utf8) else {
            throw IPCContractError.invalidUTF8
        }

        let envelope = try decoder.decode(ServerEnvelope.self, from: data)
        switch envelope.type {
        case ServerMessageType.appList:
            let response = try decoder.decode(AppListResponse.self, from: data)
            return .appList(response.apps)
        case ServerMessageType.appLaunchResult:
            let response = try decoder.decode(AppLaunchResultResponse.self, from: data)
            return .appLaunchResult(path: response.path, ok: response.ok, error: response.error)
        default:
            throw IPCContractError.unknownServerMessage(envelope.type)
        }
    }
}

enum IPCContractError: Error, Equatable {
    case invalidUTF8
    case unknownServerMessage(String)
}

private struct ServerEnvelope: Decodable {
    let type: String
}

private struct AppListResponse: Decodable {
    let type: String
    let apps: [LauncherApplication]
}

private struct AppLaunchResultResponse: Decodable {
    let type: String
    let path: String
    let ok: Bool
    let error: String?
}
