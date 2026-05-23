import Foundation

struct LauncherResult: Codable, Equatable {
    let id: String
    let title: String
    let subtitle: String?
    let source: String
    let icon: IconDescriptor?
    let actions: [LauncherAction]
}

struct LauncherAction: Codable, Equatable {
    let id: String
    let title: String
}

struct IconDescriptor: Codable, Equatable {
    let kind: String
    let value: String
}

enum ClientMessageType {
    static let query = "launcher::query"
    static let execute = "launcher::execute"
}

enum ServerMessageType {
    static let results = "launcher::results"
    static let actionResult = "launcher::action::result"
}

struct QueryRequest: Encodable {
    let type = ClientMessageType.query
    let query: String
}

struct ExecuteRequest: Encodable {
    let type = ClientMessageType.execute
    let resultID: String
    let actionID: String

    enum CodingKeys: String, CodingKey {
        case type
        case resultID = "result_id"
        case actionID = "action_id"
    }
}

enum ServerMessage: Equatable {
    case results(query: String, results: [LauncherResult])
    case actionResult(resultID: String, actionID: String, ok: Bool, error: String?)
}

struct IPCContract {
    private let decoder = JSONDecoder()
    private let encoder = JSONEncoder()

    func encodeClientLine(_ message: some Encodable) throws -> String {
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
        case ServerMessageType.results:
            let response = try decoder.decode(ResultsResponse.self, from: data)
            return .results(query: response.query, results: response.results)
        case ServerMessageType.actionResult:
            let response = try decoder.decode(ActionResultResponse.self, from: data)
            return .actionResult(
                resultID: response.resultID,
                actionID: response.actionID,
                ok: response.ok,
                error: response.error
            )
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

private struct ResultsResponse: Decodable {
    let type: String
    let query: String
    let results: [LauncherResult]
}

private struct ActionResultResponse: Decodable {
    let type: String
    let resultID: String
    let actionID: String
    let ok: Bool
    let error: String?

    enum CodingKeys: String, CodingKey {
        case type
        case resultID = "result_id"
        case actionID = "action_id"
        case ok
        case error
    }
}
