import XCTest
@testable import LaunchKick

final class LauncherStateTests: XCTestCase {
    func testReplacingNonEmptyAppsSelectsFirstApp() {
        var state = LauncherState()

        state.replaceApps(apps())

        XCTAssertEqual(state.selectedIndex, 0)
        XCTAssertEqual(state.selectedApplication(), apps()[0])
    }

    func testReplacingEmptyAppsClearsSelection() {
        var state = LauncherState()
        state.replaceApps(apps())
        state.replaceApps([])

        XCTAssertNil(state.selectedIndex)
        XCTAssertNil(state.selectedApplication())
    }

    func testMoveSelectionClampsToBounds() {
        var state = LauncherState()
        state.replaceApps(apps())

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

    private func apps() -> [LauncherApplication] {
        [
            LauncherApplication(name: "Safari", path: "/Applications/Safari.app"),
            LauncherApplication(name: "Notes", path: "/Applications/Notes.app"),
        ]
    }
}
