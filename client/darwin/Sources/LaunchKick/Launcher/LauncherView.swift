import AppKit

struct LauncherView {
    let panel: LauncherPanel
    let input: LauncherTextField
    let resultTable: NSTableView

    static func create() -> LauncherView {
        let panel = LauncherPanel(
            contentRect: NSRect(x: 0, y: 0, width: 640, height: 520),
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )

        panel.isReleasedWhenClosed = false
        panel.level = .floating
        panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
        panel.backgroundColor = .clear
        panel.isOpaque = false
        panel.hasShadow = true

        let background = NSVisualEffectView(frame: panel.contentView!.bounds)
        background.autoresizingMask = [.width, .height]
        background.material = .hudWindow
        background.blendingMode = .behindWindow
        background.state = .active
        background.wantsLayer = true
        background.layer?.cornerRadius = 18
        background.layer?.masksToBounds = true

        let input = LauncherTextField(frame: NSRect(x: 24, y: 456, width: 592, height: 40))
        input.font = NSFont.systemFont(ofSize: 24)
        input.placeholderString = "Search commands..."
        input.isBordered = false
        input.drawsBackground = false
        input.textColor = .labelColor
        input.focusRingType = .none

        let resultTable = NSTableView(frame: NSRect(x: 0, y: 0, width: 592, height: 408))
        resultTable.headerView = nil
        resultTable.rowHeight = 48
        resultTable.backgroundColor = .clear
        resultTable.selectionHighlightStyle = .regular
        resultTable.usesAlternatingRowBackgroundColors = false
        resultTable.allowsMultipleSelection = false

        let column = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("LauncherResultColumn"))
        column.width = 592
        resultTable.addTableColumn(column)

        let scrollView = NSScrollView(frame: NSRect(x: 24, y: 24, width: 592, height: 408))
        scrollView.autoresizingMask = [.width, .height]
        scrollView.drawsBackground = false
        scrollView.hasVerticalScroller = true
        scrollView.documentView = resultTable

        background.addSubview(input)
        background.addSubview(scrollView)
        panel.contentView = background
        panel.center()

        return LauncherView(panel: panel, input: input, resultTable: resultTable)
    }
}
