import AppKit

final class LauncherController: NSObject, NSTextFieldDelegate, NSTableViewDataSource, NSTableViewDelegate {
    private var panel: LauncherPanel!
    private var input: LauncherTextField!
    private var resultTable: NSTableView!
    private var state = LauncherState()
    private var hotKey: HotKey!
    private var localKeyMonitor: Any?
    private var globalKeyMonitor: Any?
    private let coreIPC: CoreIPC

    init(coreIPC: CoreIPC = CoreIPC()) {
        self.coreIPC = coreIPC
        super.init()
    }

    func start() {
        createPanel()
        listenToCore()
        coreIPC.sendQuery("")
        registerHotKey()
        registerKeyboardShortcuts()
    }

    private func createPanel() {
        let view = LauncherView.create()
        panel = view.panel
        input = view.input
        resultTable = view.resultTable

        panel.onCancel = { [weak self] in
            self?.hideLauncher()
        }
        input.onCancel = { [weak self] in
            self?.hideLauncher()
        }
        input.delegate = self
        resultTable.dataSource = self
        resultTable.delegate = self
        resultTable.target = self
        resultTable.action = #selector(resultTableClicked)
    }

    private func listenToCore() {
        coreIPC.onResults = { [weak self] _, results in
            self?.showResults(results)
        }
        coreIPC.onActionResult = { resultID, actionID, ok, error in
            guard !ok else { return }
            fputs("LaunchKick action failed for \(resultID)#\(actionID): \(error ?? "unknown error")\n", stderr)
        }
        coreIPC.startListening()
    }

    private func showResults(_ results: [LauncherResult]) {
        state.replaceResults(results)
        resultTable.reloadData()
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
                executeSelectedResult()
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
        coreIPC.sendQuery("")
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
            resultTable.deselectAll(nil)
            return
        }

        resultTable.selectRowIndexes(IndexSet(integer: selectedIndex), byExtendingSelection: false)
        resultTable.scrollRowToVisible(selectedIndex)
    }

    private func executeSelectedResult() {
        if resultTable.selectedRow != -1 {
            state.select(index: resultTable.selectedRow)
        }

        guard let intent = state.selectedExecuteIntent() else { return }

        coreIPC.sendExecute(resultID: intent.resultID, actionID: intent.actionID)
        hideLauncher()
    }

    @objc private func resultTableClicked() {
        if resultTable.clickedRow != -1 {
            state.select(index: resultTable.clickedRow)
        }
        executeSelectedResult()
    }

    func controlTextDidChange(_ notification: Notification) {
        coreIPC.sendQuery(input.stringValue)
    }

    func numberOfRows(in tableView: NSTableView) -> Int {
        state.results.count
    }

    func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
        guard let result = state.result(at: row) else { return nil }

        let cell = NSTableCellView(frame: NSRect(x: 0, y: 0, width: tableView.bounds.width, height: tableView.rowHeight))
        cell.identifier = NSUserInterfaceItemIdentifier("LauncherResultCell")

        let icon = NSImageView(frame: NSRect(x: 12, y: 8, width: 32, height: 32))
        icon.image = image(for: result)
        icon.imageScaling = .scaleProportionallyUpOrDown

        let title = NSTextField(labelWithString: result.title)
        title.frame = NSRect(x: 56, y: result.subtitle == nil ? 11 : 18, width: max(0, tableView.bounds.width - 68), height: 22)
        title.font = NSFont.systemFont(ofSize: 17, weight: .medium)
        title.textColor = .labelColor
        title.lineBreakMode = .byTruncatingTail

        cell.addSubview(icon)
        cell.addSubview(title)
        cell.imageView = icon
        cell.textField = title

        if let subtitle = result.subtitle, !subtitle.isEmpty {
            let subtitleField = NSTextField(labelWithString: subtitle)
            subtitleField.frame = NSRect(x: 56, y: 5, width: max(0, tableView.bounds.width - 68), height: 16)
            subtitleField.font = NSFont.systemFont(ofSize: 11)
            subtitleField.textColor = .secondaryLabelColor
            subtitleField.lineBreakMode = .byTruncatingMiddle
            cell.addSubview(subtitleField)
        }

        return cell
    }

    private func image(for result: LauncherResult) -> NSImage? {
        guard let icon = result.icon else {
            return NSImage(systemSymbolName: "command", accessibilityDescription: nil)
        }

        switch icon.kind {
        case "file":
            return NSWorkspace.shared.icon(forFile: icon.value)
        default:
            return NSImage(systemSymbolName: "command", accessibilityDescription: nil)
        }
    }
}
