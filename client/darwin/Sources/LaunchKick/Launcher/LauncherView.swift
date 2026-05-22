import AppKit

struct LauncherView {
    let panel: LauncherPanel
    let input: LauncherTextField
    let resultLabel: NSTextField

    static func create() -> LauncherView {
        let panel = LauncherPanel(
            contentRect: NSRect(x: 0, y: 0, width: 640, height: 130),
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

        let input = LauncherTextField(frame: NSRect(x: 24, y: 66, width: 592, height: 40))
        input.font = NSFont.systemFont(ofSize: 24)
        input.placeholderString = "Search or type a command..."
        input.isBordered = false
        input.drawsBackground = false
        input.textColor = .labelColor
        input.focusRingType = .none

        let resultLabel = NSTextField(labelWithString: "Result: ")
        resultLabel.frame = NSRect(x: 24, y: 24, width: 592, height: 24)
        resultLabel.font = NSFont.systemFont(ofSize: 18)
        resultLabel.textColor = .secondaryLabelColor

        background.addSubview(input)
        background.addSubview(resultLabel)
        panel.contentView = background
        panel.center()

        return LauncherView(panel: panel, input: input, resultLabel: resultLabel)
    }
}
