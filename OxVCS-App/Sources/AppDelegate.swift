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
        alert.messageText = "OxVCS"
        alert.informativeText = "Logic Pro Version Control\nPowered by Oxen\n\nVersion 1.0.0"
        alert.alertStyle = .informational
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }

    @objc func showSettings() {
        let settingsWindow = SettingsWindow()
        settingsWindow.show()
    }

    @objc func showProjectWizard() {
        let wizard = ProjectWizardWindow()
        wizard.show()
    }

    @objc func showMergeHelperFromMenu() {
        NotificationCenter.default.post(name: .showMergeHelper, object: nil)
    }

    @objc private func showHelp() {
        let alert = NSAlert()
        alert.messageText = "OxVCS for Logic Pro"
        alert.informativeText = """
        Version Control for Logic Pro Projects

        Features:
        • Automatic background tracking
        • Milestone commits with metadata
        • Rollback to any previous version
        • File locking for collaboration
        • Manual merge helpers

        Documentation:
        See README.md and docs/ folder in the repository

        Support:
        https://github.com/oxen-ai/oxen-vcs-logic
        """
        alert.alertStyle = .informational
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }
}
