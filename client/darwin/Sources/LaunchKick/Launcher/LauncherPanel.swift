import AppKit

final class LauncherPanel: NSPanel {
    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { true }

    override func keyDown(with event: NSEvent) {
        guard !event.isEscape else {
            orderOut(nil)
            return
        }

        super.keyDown(with: event)
    }
}

final class LauncherTextField: NSTextField {
    override func cancelOperation(_ sender: Any?) {
        window?.orderOut(nil)
    }

    override func keyDown(with event: NSEvent) {
        guard !event.isEscape else {
            window?.orderOut(nil)
            return
        }

        super.keyDown(with: event)
    }
}

extension NSEvent {
    var isEscape: Bool { keyCode == 53 }
}
