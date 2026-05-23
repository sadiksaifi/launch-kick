import AppKit

final class AppDelegate: NSObject, NSApplicationDelegate, NSTextFieldDelegate, NSTableViewDataSource, NSTableViewDelegate {
    private var panel: LauncherPanel!
    private var input: LauncherTextField!
    private var appTable: NSTableView!
    private var state = LauncherState()
    private var hotKey: HotKey!
    private var localKeyMonitor: Any?
    private var globalKeyMonitor: Any?
    private let coreIPC = CoreIPC()

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.accessory)
        createPanel()
        listenToCore()
        coreIPC.sendAppListRequest()
        registerHotKey()
        registerKeyboardShortcuts()
    }

    private func createPanel() {
        let view = LauncherView.create()
        panel = view.panel
        input = view.input
        appTable = view.appTable
        input.delegate = self
        appTable.dataSource = self
        appTable.delegate = self
        appTable.target = self
        appTable.action = #selector(appTableClicked)
    }

    private func listenToCore() {
        coreIPC.onAppList = { [weak self] apps in
            self?.showApps(apps)
        }
        coreIPC.onAppLaunchResult = { path, ok, error in
            guard !ok else { return }
            fputs("LaunchKick launch failed for \(path): \(error ?? "unknown error")\n", stderr)
        }
        coreIPC.startListening()
    }

    private func showApps(_ apps: [LauncherApplication]) {
        state.replaceApps(apps)
        appTable.reloadData()
        syncSelectionToTable()
    }

    private func registerHotKey() {
        hotKey = HotKey { [weak self] in
            self?.toggleLauncher()
        }
        hotKey.register()
    }

    private func registerKeyboardShortcuts() {
        localKeyMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self] event in
            guard let self, panel.isVisible else {
                return event
            }

            if event.isEscape {
                hideLauncher()
                return nil
            }

            if event.isReturnOrEnter {
                launchSelectedApp()
                return nil
            }

            if event.isArrowDown {
                moveSelection(by: 1)
                return nil
            }

            if event.isArrowUp {
                moveSelection(by: -1)
                return nil
            }

            return event
        }

        globalKeyMonitor = NSEvent.addGlobalMonitorForEvents(matching: .keyDown) { [weak self] event in
            guard event.isEscape, self?.panel.isVisible == true else { return }

            DispatchQueue.main.async {
                self?.hideLauncher()
            }
        }
    }

    private func toggleLauncher() {
        if panel.isVisible {
            hideLauncher()
            return
        }

        showLauncher()
    }

    private func showLauncher() {
        state.show()
        panel.center()
        panel.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
        input.stringValue = ""
        syncSelectionToTable()
        panel.makeFirstResponder(input)
        coreIPC.sendAppListRequest()
    }

    private func hideLauncher() {
        state.hide()
        panel.orderOut(nil)
    }

    private func moveSelection(by delta: Int) {
        state.moveSelection(by: delta)
        syncSelectionToTable()
    }

    private func syncSelectionToTable() {
        guard let selectedIndex = state.selectedIndex else {
            appTable.deselectAll(nil)
            return
        }

        appTable.selectRowIndexes(IndexSet(integer: selectedIndex), byExtendingSelection: false)
        appTable.scrollRowToVisible(selectedIndex)
    }

    private func launchSelectedApp() {
        if appTable.selectedRow != -1 {
            state.select(index: appTable.selectedRow)
        }

        guard let app = state.selectedApplication() else { return }

        coreIPC.sendAppLaunch(path: app.path)
        hideLauncher()
    }

    @objc private func appTableClicked() {
        if appTable.clickedRow != -1 {
            state.select(index: appTable.clickedRow)
        }
        launchSelectedApp()
    }

    func numberOfRows(in tableView: NSTableView) -> Int {
        state.apps.count
    }

    func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
        guard let app = state.app(at: row) else { return nil }

        let cell = NSTableCellView(frame: NSRect(x: 0, y: 0, width: tableView.bounds.width, height: tableView.rowHeight))
        cell.identifier = NSUserInterfaceItemIdentifier("ApplicationCell")

        let icon = NSImageView(frame: NSRect(x: 12, y: 8, width: 32, height: 32))
        icon.image = NSWorkspace.shared.icon(forFile: app.path)
        icon.imageScaling = .scaleProportionallyUpOrDown

        let name = NSTextField(labelWithString: app.name)
        name.frame = NSRect(x: 56, y: 11, width: max(0, tableView.bounds.width - 68), height: 26)
        name.font = NSFont.systemFont(ofSize: 17, weight: .medium)
        name.textColor = .labelColor
        name.lineBreakMode = .byTruncatingTail

        cell.addSubview(icon)
        cell.addSubview(name)
        cell.imageView = icon
        cell.textField = name
        return cell
    }
}
