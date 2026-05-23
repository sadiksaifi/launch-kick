@testable import LaunchKick
import XCTest

final class LauncherInteractionTests: XCTestCase {
    func testStartedSendsEmptyQuery() {
        var interaction = LauncherInteraction()

        XCTAssertEqual(interaction.apply(.started), [.sendToCore(.queryChanged(""))])
    }

    func testToggleVisibilityShowsLauncherAndRefreshesQuery() {
        var interaction = LauncherInteraction()

        let effects = interaction.apply(.toggleVisibility)

        XCTAssertTrue(interaction.stateSnapshot.isVisible)
        XCTAssertEqual(
            effects,
            [.showPanel, .clearInput, .syncSelection, .focusInput, .sendToCore(.queryChanged(""))]
        )
    }

    func testToggleVisibilityHidesVisibleLauncher() {
        var interaction = LauncherInteraction()
        _ = interaction.apply(.toggleVisibility)

        XCTAssertEqual(interaction.apply(.toggleVisibility), [.hidePanel])
        XCTAssertFalse(interaction.stateSnapshot.isVisible)
    }

    func testQueryChangeSendsQueryToCore() {
        var interaction = LauncherInteraction()

        XCTAssertEqual(interaction.apply(.queryChanged("saf")), [.sendToCore(.queryChanged("saf"))])
    }

    func testIncomingResultsReplaceStateAndReload() {
        var interaction = LauncherInteraction()

        let effects = interaction.receive(.results(query: "", results: results()))

        XCTAssertEqual(interaction.stateSnapshot.results, results())
        XCTAssertEqual(interaction.stateSnapshot.selectedIndex, 0)
        XCTAssertEqual(effects, [.reloadResults, .syncSelection])
    }

    func testSelectionMovementUpdatesStateAndSyncsSelection() {
        var interaction = LauncherInteraction()
        _ = interaction.receive(.results(query: "", results: results()))

        XCTAssertEqual(interaction.apply(.moveSelection(1)), [.syncSelection])
        XCTAssertEqual(interaction.stateSnapshot.selectedIndex, 1)
    }

    func testIncomingResultsResetSelectionToFirstResult() {
        var interaction = LauncherInteraction()
        _ = interaction.receive(.results(query: "", results: results()))
        _ = interaction.apply(.selectResult(index: 1))

        let effects = interaction.receive(.results(query: "s", results: results()))

        XCTAssertEqual(interaction.stateSnapshot.selectedIndex, 0)
        XCTAssertEqual(interaction.stateSnapshot.selectedResult()?.id, "application:/Applications/Safari.app")
        XCTAssertEqual(effects, [.reloadResults, .syncSelection])
    }

    func testExecuteSelectedSendsExecuteIntentAndHidesLauncher() {
        var interaction = LauncherInteraction()
        _ = interaction.apply(.toggleVisibility)
        _ = interaction.receive(.results(query: "", results: results()))
        _ = interaction.apply(.selectResult(index: 1))

        let effects = interaction.apply(.executeSelected)

        XCTAssertEqual(
            effects,
            [
                .sendToCore(.execute(ExecuteIntent(resultID: "application:/Applications/Notes.app", actionID: "open"))),
                .hidePanel,
            ]
        )
        XCTAssertFalse(interaction.stateSnapshot.isVisible)
    }

    func testExecuteSelectedWithoutActionDoesNothing() {
        var interaction = LauncherInteraction()
        _ = interaction.receive(.results(query: "", results: [result(id: "command:empty", title: "Empty", actions: [])]))

        XCTAssertEqual(interaction.apply(.executeSelected), [])
    }

    func testFailedActionResultLogsFailure() {
        var interaction = LauncherInteraction()
        let intent = ExecuteIntent(resultID: "application:/Applications/Missing.app", actionID: "open")

        XCTAssertEqual(
            interaction.receive(.actionResult(intent: intent, ok: false, error: "launch failed")),
            [.logError("LaunchKick action failed for application:/Applications/Missing.app#open: launch failed")]
        )
    }

    func testSuccessfulActionResultHasNoEffect() {
        var interaction = LauncherInteraction()
        let intent = ExecuteIntent(resultID: "application:/Applications/Safari.app", actionID: "open")

        XCTAssertEqual(interaction.receive(.actionResult(intent: intent, ok: true, error: nil)), [])
    }

    private func results() -> [LauncherResult] {
        [
            result(id: "application:/Applications/Safari.app", title: "Safari"),
            result(id: "application:/Applications/Notes.app", title: "Notes"),
        ]
    }

    private func result(id: String, title: String, actions: [LauncherAction] = [LauncherAction(id: "open", title: "Open")]) -> LauncherResult {
        LauncherResult(
            id: id,
            title: title,
            subtitle: nil,
            source: "applications",
            icon: nil,
            actions: actions
        )
    }
}
