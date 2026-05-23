import AppKit

final class AppDelegate: NSObject, NSApplicationDelegate, NSTextFieldDelegate, NSTableViewDataSource, NSTableViewDelegate {
    private var panel: LauncherPanel!
    private var input: LauncherTextField!
    private var appTable: NSTableView!
    private var apps: [LauncherApplication] = []
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
        coreIPC.startListening()
    }

    private func showApps(_ apps: [LauncherApplication]) {
        self.apps = apps
        appTable.reloadData()

        if !apps.isEmpty && appTable.selectedRow == -1 {
            appTable.selectRowIndexes(IndexSet(integer: 0), byExtendingSelection: false)
        }
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

        panel.center()
        panel.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
        input.stringValue = ""
        ensureSelection()
        panel.makeFirstResponder(input)
        coreIPC.sendAppListRequest()
    }

    private func hideLauncher() {
        panel.orderOut(nil)
    }

    private func ensureSelection() {
        guard !apps.isEmpty, appTable.selectedRow == -1 else { return }
        appTable.selectRowIndexes(IndexSet(integer: 0), byExtendingSelection: false)
    }

    private func moveSelection(by delta: Int) {
        guard !apps.isEmpty else { return }

        let currentRow = appTable.selectedRow == -1 ? 0 : appTable.selectedRow
        let nextRow = max(0, min(apps.count - 1, currentRow + delta))
        appTable.selectRowIndexes(IndexSet(integer: nextRow), byExtendingSelection: false)
        appTable.scrollRowToVisible(nextRow)
    }

    private func launchSelectedApp() {
        let selectedRow = appTable.selectedRow
        guard apps.indices.contains(selectedRow) else { return }

        coreIPC.sendAppLaunch(path: apps[selectedRow].path)
        hideLauncher()
    }

    @objc private func appTableClicked() {
        launchSelectedApp()
    }

    func numberOfRows(in tableView: NSTableView) -> Int {
        apps.count
    }

    func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
        guard apps.indices.contains(row) else { return nil }

        let app = apps[row]
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
