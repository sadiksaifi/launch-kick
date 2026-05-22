import AppKit

final class AppDelegate: NSObject, NSApplicationDelegate, NSTextFieldDelegate {
    private var panel: LauncherPanel!
    private var input: LauncherTextField!
    private var resultLabel: NSTextField!
    private var hotKey: HotKey!
    private var localKeyMonitor: Any?
    private var globalKeyMonitor: Any?
    private let coreIPC = CoreIPC()

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.accessory)
        createPanel()
        listenToCore()
        registerHotKey()
        registerEscapeKey()
    }

    private func createPanel() {
        let view = LauncherView.create()
        panel = view.panel
        input = view.input
        resultLabel = view.resultLabel
        input.delegate = self
    }

    private func listenToCore() {
        coreIPC.onResult = { [weak self] value in
            self?.resultLabel.stringValue = "Result: \(value)"
        }
        coreIPC.startListening()
    }

    private func registerHotKey() {
        hotKey = HotKey { [weak self] in
            self?.toggleLauncher()
        }
        hotKey.register()
    }

    private func registerEscapeKey() {
        localKeyMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self] event in
            guard event.isEscape, self?.panel.isVisible == true else {
                return event
            }

            self?.hideLauncher()
            return nil
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
        resultLabel.stringValue = "Result: "
        panel.makeFirstResponder(input)
    }

    private func hideLauncher() {
        panel.orderOut(nil)
    }

    func controlTextDidChange(_ notification: Notification) {
        coreIPC.sendInput(input.stringValue)
    }
}
