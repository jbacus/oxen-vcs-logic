import AppKit
import SwiftUI

class AppDelegate: NSObject, NSApplicationDelegate {
    var selectedProject: Project?

    func applicationDidFinishLaunching(_ notification: Notification) {
        // SwiftUI handles window creation
    }

    func applicationWillTerminate(_ notification: Notification) {
        // Cleanup
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        return true
    }


    @objc func showAbout() {
        let alert = NSAlert()
        alert.messageText = "Auxin"
        alert.informativeText = "Version Control for Creative Applications\nPowered by Oxen\n\nSupports:\n• Logic Pro (.logicx)\n• SketchUp (.skp)\n• Blender (.blend)\n\nVersion 1.0.0"
        alert.alertStyle = .informational
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }

    @objc func showSettings() {
        let settingsWindow = SettingsWindow()
        settingsWindow.show()
    }

    @objc func showProjectWizard() {
        NotificationCenter.default.post(name: .showProjectWizard, object: nil)
    }

    @objc func showMergeHelperFromMenu() {
        NotificationCenter.default.post(name: .showMergeHelper, object: nil)
    }

    @objc private func showHelp() {
        let alert = NSAlert()
        alert.messageText = "Auxin"
        alert.informativeText = """
        Version Control for Creative Applications

        Supported Applications:
        • Logic Pro (.logicx) - Audio production
        • SketchUp (.skp) - 3D modeling
        • Blender (.blend) - 3D animation

        Features:
        • Automatic background tracking
        • Milestone commits with metadata
        • Rollback to any previous version
        • File locking for collaboration
        • Manual merge helpers

        Documentation:
        See README.md and docs/ folder in the repository

        Support:
        https://github.com/jbacus/auxin
        """
        alert.alertStyle = .informational
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }
}
