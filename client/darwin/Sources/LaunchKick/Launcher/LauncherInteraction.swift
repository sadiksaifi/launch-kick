enum LauncherUserIntent: Equatable {
    case started
    case toggleVisibility
    case hide
    case queryChanged(String)
    case moveSelection(Int)
    case selectResult(index: Int)
    case executeSelected
}

enum LauncherCoreEvent: Equatable {
    case results(query: String, results: [LauncherResult])
    case actionResult(intent: ExecuteIntent, ok: Bool, error: String?)
}

enum LauncherEffect: Equatable {
    case showPanel
    case hidePanel
    case focusInput
    case clearInput
    case reloadResults
    case syncSelection
    case sendToCore(CoreIPCIntent)
    case logError(String)
}

struct LauncherInteraction {
    private var state = LauncherState()

    var stateSnapshot: LauncherState {
        state
    }

    mutating func apply(_ intent: LauncherUserIntent) -> [LauncherEffect] {
        switch intent {
        case .started:
            return [.sendToCore(.queryChanged(""))]
        case .toggleVisibility:
            return state.isVisible ? hide() : show()
        case .hide:
            return hide()
        case let .queryChanged(query):
            return [.sendToCore(.queryChanged(query))]
        case let .moveSelection(delta):
            state.moveSelection(by: delta)
            return [.syncSelection]
        case let .selectResult(index):
            state.select(index: index)
            return [.syncSelection]
        case .executeSelected:
            guard let intent = state.selectedExecuteIntent() else { return [] }
            state.hide()
            return [.sendToCore(.execute(intent)), .hidePanel]
        }
    }

    mutating func receive(_ event: LauncherCoreEvent) -> [LauncherEffect] {
        switch event {
        case let .results(_, results):
            state.replaceResults(results)
            return [.reloadResults, .syncSelection]
        case let .actionResult(intent, ok, error):
            guard !ok else { return [] }
            return [.logError("LaunchKick action failed for \(intent.resultID)#\(intent.actionID): \(error ?? "unknown error")")]
        }
    }

    func result(at index: Int) -> LauncherResult? {
        state.result(at: index)
    }

    private mutating func show() -> [LauncherEffect] {
        state.show()
        return [
            .showPanel,
            .clearInput,
            .syncSelection,
            .focusInput,
            .sendToCore(.queryChanged("")),
        ]
    }

    private mutating func hide() -> [LauncherEffect] {
        state.hide()
        return [.hidePanel]
    }
}
