@testable import LaunchKick
import XCTest

final class LauncherStateTests: XCTestCase {
    func testReplacingNonEmptyResultsSelectsFirstResult() {
        var state = LauncherState()

        state.replaceResults(results())

        XCTAssertEqual(state.selectedIndex, 0)
        XCTAssertEqual(state.selectedResult(), results()[0])
    }

    func testReplacingEmptyResultsClearsSelection() {
        var state = LauncherState()
        state.replaceResults(results())
        state.replaceResults([])

        XCTAssertNil(state.selectedIndex)
        XCTAssertNil(state.selectedResult())
    }

    func testReplacingResultsPreservesSelectionByResultID() {
        var state = LauncherState()
        state.replaceResults(results())
        state.select(index: 1)

        state.replaceResults([results()[1], results()[0]])

        XCTAssertEqual(state.selectedIndex, 0)
        XCTAssertEqual(state.selectedResult()?.id, "application:/Applications/Notes.app")
    }

    func testMoveSelectionClampsToBounds() {
        var state = LauncherState()
        state.replaceResults(results())

        state.moveSelection(by: 10)
        XCTAssertEqual(state.selectedIndex, 1)

        state.moveSelection(by: -10)
        XCTAssertEqual(state.selectedIndex, 0)
    }

    func testShowHideAndToggleUpdateVisibility() {
        var state = LauncherState()

        state.show()
        XCTAssertTrue(state.isVisible)

        state.hide()
        XCTAssertFalse(state.isVisible)

        state.toggleVisibility()
        XCTAssertTrue(state.isVisible)
    }

    func testSelectedExecuteIntentUsesSelectedResultFirstAction() {
        var state = LauncherState()
        state.replaceResults(results())
        state.select(index: 1)

        XCTAssertEqual(
            state.selectedExecuteIntent(),
            ExecuteIntent(resultID: "application:/Applications/Notes.app", actionID: "open")
        )
    }

    func testSelectedExecuteIntentIsNilWithoutASelectedResult() {
        let state = LauncherState()

        XCTAssertNil(state.selectedExecuteIntent())
    }

    func testSelectedExecuteIntentIsNilWhenResultHasNoActions() {
        var state = LauncherState()
        state.replaceResults([
            LauncherResult(
                id: "command:empty",
                title: "Empty",
                subtitle: nil,
                source: "test",
                icon: nil,
                actions: []
            ),
        ])

        XCTAssertNil(state.selectedExecuteIntent())
    }

    private func results() -> [LauncherResult] {
        [
            LauncherResult(
                id: "application:/Applications/Safari.app",
                title: "Safari",
                subtitle: "/Applications/Safari.app",
                source: "applications",
                icon: IconDescriptor(kind: "file", value: "/Applications/Safari.app"),
                actions: [LauncherAction(id: "open", title: "Open")]
            ),
            LauncherResult(
                id: "application:/Applications/Notes.app",
                title: "Notes",
                subtitle: "/Applications/Notes.app",
                source: "applications",
                icon: IconDescriptor(kind: "file", value: "/Applications/Notes.app"),
                actions: [LauncherAction(id: "open", title: "Open")]
            ),
        ]
    }
}
