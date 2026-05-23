import AppKit

final class AppDelegate: NSObject, NSApplicationDelegate {
    private var launcherController: LauncherController!

    func applicationDidFinishLaunching(_: Notification) {
        NSApp.setActivationPolicy(.accessory)
        launcherController = LauncherController()
        launcherController.start()
    }
}
