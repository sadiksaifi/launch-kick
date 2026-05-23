import AppKit

final class LauncherController: NSObject, NSTextFieldDelegate, NSTableViewDataSource, NSTableViewDelegate {
    private var panel: LauncherPanel!
    private var input: LauncherTextField!
    private var resultTable: NSTableView!
    private var interaction = LauncherInteraction()
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
        perform(interaction.apply(.started))
        registerHotKey()
        registerKeyboardShortcuts()
    }

    private func createPanel() {
        let view = LauncherView.create()
        panel = view.panel
        input = view.input
        resultTable = view.resultTable

        panel.onCancel = { [weak self] in
            self?.apply(.hide)
        }
        panel.onFocusLost = { [weak self] in
            guard let self, interaction.stateSnapshot.isVisible else { return }
            apply(.hide)
        }
        input.onCancel = { [weak self] in
            self?.apply(.hide)
        }
        input.delegate = self
        resultTable.dataSource = self
        resultTable.delegate = self
        resultTable.target = self
        resultTable.action = #selector(resultTableClicked)
    }

    private func listenToCore() {
        coreIPC.onEvent = { [weak self] event in
            self?.receive(event)
        }
        coreIPC.startListening()
    }

    private func receive(_ event: CoreIPCEvent) {
        switch event {
        case let .results(query, results):
            perform(interaction.receive(.results(query: query, results: results)))
        case let .actionResult(intent, ok, error):
            perform(interaction.receive(.actionResult(intent: intent, ok: ok, error: error)))
        case let .failed(error):
            perform([.logError("LaunchKick IPC failed: \(error)")])
        }
    }

    private func registerHotKey() {
        hotKey = HotKey { [weak self] in
            self?.apply(.toggleVisibility)
        }
        hotKey.register()
    }

    private func registerKeyboardShortcuts() {
        localKeyMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self] event in
            guard let self, panel.isVisible else {
                return event
            }

            if event.isEscape {
                apply(.hide)
                return nil
            }

            if event.isReturnOrEnter {
                executeSelectedResult()
                return nil
            }

            if event.isArrowDown {
                apply(.moveSelection(1))
                return nil
            }

            if event.isArrowUp {
                apply(.moveSelection(-1))
                return nil
            }

            return event
        }

        globalKeyMonitor = NSEvent.addGlobalMonitorForEvents(matching: .keyDown) { [weak self] event in
            guard event.isEscape, self?.panel.isVisible == true else { return }

            DispatchQueue.main.async {
                self?.apply(.hide)
            }
        }
    }

    private func apply(_ intent: LauncherUserIntent) {
        perform(interaction.apply(intent))
    }

    private func perform(_ effects: [LauncherEffect]) {
        for effect in effects {
            switch effect {
            case .showPanel:
                panel.center()
                panel.makeKeyAndOrderFront(nil)
                NSApp.activate(ignoringOtherApps: true)
            case .hidePanel:
                panel.orderOut(nil)
            case .focusInput:
                panel.makeFirstResponder(input)
            case .clearInput:
                input.stringValue = ""
            case .reloadResults:
                resultTable.reloadData()
            case .syncSelection:
                syncSelectionToTable()
            case let .sendToCore(intent):
                coreIPC.send(intent)
            case let .logError(message):
                fputs("\(message)\n", stderr)
            }
        }
    }

    private func syncSelectionToTable() {
        guard let selectedIndex = interaction.stateSnapshot.selectedIndex else {
            resultTable.deselectAll(nil)
            return
        }

        resultTable.selectRowIndexes(IndexSet(integer: selectedIndex), byExtendingSelection: false)
        resultTable.scrollRowToVisible(selectedIndex)
    }

    private func executeSelectedResult() {
        if resultTable.selectedRow != -1 {
            apply(.selectResult(index: resultTable.selectedRow))
        }
        apply(.executeSelected)
    }

    @objc private func resultTableClicked() {
        if resultTable.clickedRow != -1 {
            apply(.selectResult(index: resultTable.clickedRow))
        }
        executeSelectedResult()
    }

    func controlTextDidChange(_: Notification) {
        apply(.queryChanged(input.stringValue))
    }

    func numberOfRows(in _: NSTableView) -> Int {
        interaction.stateSnapshot.results.count
    }

    func tableView(_ tableView: NSTableView, viewFor _: NSTableColumn?, row: Int) -> NSView? {
        guard let result = interaction.result(at: row) else { return nil }

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
