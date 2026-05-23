struct LauncherState: Equatable {
    private(set) var apps: [LauncherApplication] = []
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
        isVisible ? hide() : show()
    }

    mutating func replaceApps(_ apps: [LauncherApplication]) {
        self.apps = apps

        guard !apps.isEmpty else {
            selectedIndex = nil
            return
        }

        if let selectedIndex, apps.indices.contains(selectedIndex) {
            return
        }

        selectedIndex = 0
    }

    mutating func moveSelection(by delta: Int) {
        guard !apps.isEmpty else {
            selectedIndex = nil
            return
        }

        let currentIndex = selectedIndex ?? 0
        selectedIndex = max(0, min(apps.count - 1, currentIndex + delta))
    }

    mutating func select(index: Int) {
        selectedIndex = apps.indices.contains(index) ? index : nil
    }

    func selectedApplication() -> LauncherApplication? {
        guard let selectedIndex, apps.indices.contains(selectedIndex) else { return nil }
        return apps[selectedIndex]
    }

    func app(at index: Int) -> LauncherApplication? {
        guard apps.indices.contains(index) else { return nil }
        return apps[index]
    }

    private mutating func ensureSelection() {
        if !apps.isEmpty && selectedIndex == nil {
            selectedIndex = 0
        }
    }
}
