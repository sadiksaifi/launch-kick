struct ExecuteIntent: Equatable {
    let resultID: String
    let actionID: String
}

struct LauncherState: Equatable {
    private(set) var results: [LauncherResult] = []
    private(set) var selectedIndex: Int?
    private(set) var isVisible = false

    mutating func show() {
        isVisible = true
        ensureSelection()
    }

    mutating func hide() {
        isVisible = false
    }

    mutating func toggleVisibility() {
        if isVisible {
            hide()
        } else {
            show()
        }
    }

    mutating func replaceResults(_ results: [LauncherResult]) {
        self.results = results

        guard !results.isEmpty else {
            selectedIndex = nil
            return
        }

        selectedIndex = 0
    }

    mutating func moveSelection(by delta: Int) {
        guard !results.isEmpty else {
            selectedIndex = nil
            return
        }

        let currentIndex = selectedIndex ?? 0
        selectedIndex = max(0, min(results.count - 1, currentIndex + delta))
    }

    mutating func select(index: Int) {
        selectedIndex = results.indices.contains(index) ? index : nil
    }

    func selectedResult() -> LauncherResult? {
        guard let selectedIndex, results.indices.contains(selectedIndex) else { return nil }
        return results[selectedIndex]
    }

    func result(at index: Int) -> LauncherResult? {
        guard results.indices.contains(index) else { return nil }
        return results[index]
    }

    private mutating func ensureSelection() {
        if !results.isEmpty, selectedIndex == nil {
            selectedIndex = 0
        }
    }
}
