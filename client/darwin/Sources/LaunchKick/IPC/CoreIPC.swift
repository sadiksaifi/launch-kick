import Foundation

enum CoreIPCError: Error, Equatable {
    case invalidUTF8Line
    case invalidServerMessage(String)
    case clientEncodingFailed
}

enum CoreIPCIntent: Equatable {
    case queryChanged(String)
    case execute(ExecuteIntent)
}

enum CoreIPCEvent: Equatable {
    case results(query: String, results: [LauncherResult])
    case actionResult(intent: ExecuteIntent, ok: Bool, error: String?)
    case failed(CoreIPCError)
}

final class CoreIPC {
    private let contract: IPCContract
    private let stream: CoreIPCStream

    var onEvent: ((CoreIPCEvent) -> Void)?

    init(
        input: FileHandle = .standardInput,
        output: FileHandle = .standardOutput,
        callbackQueue: DispatchQueue = .main,
        contract: IPCContract = IPCContract()
    ) {
        stream = CoreIPCStream(input: input, output: output, callbackQueue: callbackQueue)
        self.contract = contract
    }

    deinit {
        stopListening()
    }

    func startListening() {
        stream.start { [weak self] event in
            self?.handle(event)
        }
    }

    func stopListening() {
        stream.stop()
    }

    func send(_ intent: CoreIPCIntent) {
        let message: ClientMessage = switch intent {
        case let .queryChanged(query):
            .query(query)
        case let .execute(intent):
            .execute(resultID: intent.resultID, actionID: intent.actionID)
        }

        do {
            let line = try contract.encodeClientLine(message)
            try stream.writeLine(line)
        } catch {
            onEvent?(.failed(.clientEncodingFailed))
        }
    }

    private func handle(_ event: CoreIPCStreamEvent) {
        switch event {
        case let .line(line):
            handleLine(line)
        case .invalidUTF8Line:
            onEvent?(.failed(.invalidUTF8Line))
        case .closed:
            break
        }
    }

    private func handleLine(_ line: String) {
        do {
            let message = try contract.decodeServerLine(line)
            switch message {
            case let .results(query, results):
                onEvent?(.results(query: query, results: results))
            case let .actionResult(resultID, actionID, ok, error):
                onEvent?(.actionResult(
                    intent: ExecuteIntent(resultID: resultID, actionID: actionID),
                    ok: ok,
                    error: error
                ))
            }
        } catch {
            onEvent?(.failed(.invalidServerMessage(line)))
        }
    }
}
