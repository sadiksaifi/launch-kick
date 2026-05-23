import AppKit

final class LauncherPanel: NSPanel {
    var onCancel: (() -> Void)?
    var onFocusLost: (() -> Void)?

    override var canBecomeKey: Bool {
        true
    }

    override var canBecomeMain: Bool {
        true
    }

    override func keyDown(with event: NSEvent) {
        guard !event.isEscape else {
            onCancel?()
            return
        }

        super.keyDown(with: event)
    }

    override func resignKey() {
        super.resignKey()
        onFocusLost?()
    }
}

final class LauncherTextField: NSTextField {
    var onCancel: (() -> Void)?

    override func cancelOperation(_: Any?) {
        onCancel?()
    }

    override func keyDown(with event: NSEvent) {
        guard !event.isEscape else {
            onCancel?()
            return
        }

        super.keyDown(with: event)
    }
}

extension NSEvent {
    var isEscape: Bool {
        keyCode == 53
    }

    var isReturnOrEnter: Bool {
        keyCode == 36 || keyCode == 76
    }

    var isArrowDown: Bool {
        keyCode == 125
    }

    var isArrowUp: Bool {
        keyCode == 126
    }

    var isControlN: Bool {
        let flags = modifierFlags.intersection(.deviceIndependentFlagsMask)
        return flags == .control && charactersIgnoringModifiers == "n"
    }

    var isControlP: Bool {
        let flags = modifierFlags.intersection(.deviceIndependentFlagsMask)
        return flags == .control && charactersIgnoringModifiers == "p"
    }
}
