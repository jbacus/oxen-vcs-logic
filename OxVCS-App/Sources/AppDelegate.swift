import AppKit

class AppDelegate: NSObject, NSApplicationDelegate {
    var mainWindow: NSWindow?
    var mainViewController: MainViewController?

    func applicationDidFinishLaunching(_ notification: Notification) {
        setupMenuBar()
        setupMainWindow()
    }

    func applicationWillTerminate(_ notification: Notification) {
        // Cleanup
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        return true
    }

    private func setupMenuBar() {
        let mainMenu = NSMenu()

        // App Menu
        let appMenuItem = NSMenuItem()
        mainMenu.addItem(appMenuItem)
        let appMenu = NSMenu()
        appMenuItem.submenu = appMenu

        appMenu.addItem(NSMenuItem(title: "About OxVCS", action: #selector(showAbout), keyEquivalent: ""))
        appMenu.addItem(NSMenuItem.separator())
        appMenu.addItem(NSMenuItem(title: "Preferences...", action: #selector(showSettings), keyEquivalent: ","))
        appMenu.addItem(NSMenuItem.separator())
        appMenu.addItem(NSMenuItem(title: "Quit OxVCS", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q"))

        // File Menu
        let fileMenuItem = NSMenuItem()
        mainMenu.addItem(fileMenuItem)
        let fileMenu = NSMenu(title: "File")
        fileMenuItem.submenu = fileMenu

        fileMenu.addItem(NSMenuItem(title: "Initialize New Project...", action: #selector(showProjectWizard), keyEquivalent: "n"))
        fileMenu.addItem(NSMenuItem.separator())
        fileMenu.addItem(NSMenuItem(title: "Close Window", action: #selector(NSWindow.performClose(_:)), keyEquivalent: "w"))

        // Edit Menu
        let editMenuItem = NSMenuItem()
        mainMenu.addItem(editMenuItem)
        let editMenu = NSMenu(title: "Edit")
        editMenuItem.submenu = editMenu

        editMenu.addItem(NSMenuItem(title: "Cut", action: #selector(NSText.cut(_:)), keyEquivalent: "x"))
        editMenu.addItem(NSMenuItem(title: "Copy", action: #selector(NSText.copy(_:)), keyEquivalent: "c"))
        editMenu.addItem(NSMenuItem(title: "Paste", action: #selector(NSText.paste(_:)), keyEquivalent: "v"))

        // View Menu
        let viewMenuItem = NSMenuItem()
        mainMenu.addItem(viewMenuItem)
        let viewMenu = NSMenu(title: "View")
        viewMenuItem.submenu = viewMenu

        viewMenu.addItem(NSMenuItem(title: "Refresh Project List", action: #selector(refreshProjectList), keyEquivalent: "r"))
        viewMenu.addItem(NSMenuItem.separator())
        viewMenu.addItem(NSMenuItem(title: "Merge Helper...", action: #selector(showMergeHelper), keyEquivalent: ""))

        // Window Menu
        let windowMenuItem = NSMenuItem()
        mainMenu.addItem(windowMenuItem)
        let windowMenu = NSMenu(title: "Window")
        windowMenuItem.submenu = windowMenu

        windowMenu.addItem(NSMenuItem(title: "Minimize", action: #selector(NSWindow.miniaturize(_:)), keyEquivalent: "m"))
        windowMenu.addItem(NSMenuItem(title: "Zoom", action: #selector(NSWindow.zoom(_:)), keyEquivalent: ""))

        // Help Menu
        let helpMenuItem = NSMenuItem()
        mainMenu.addItem(helpMenuItem)
        let helpMenu = NSMenu(title: "Help")
        helpMenuItem.submenu = helpMenu

        helpMenu.addItem(NSMenuItem(title: "OxVCS Help", action: #selector(showHelp), keyEquivalent: "?"))

        NSApp.mainMenu = mainMenu
    }

    private func setupMainWindow() {
        // Create window
        let window = NSWindow(
            contentRect: NSRect(x: 100, y: 100, width: 1200, height: 800),
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: false
        )

        window.title = "OxVCS - Logic Pro Version Control"
        window.minSize = NSSize(width: 800, height: 600)
        window.backgroundColor = .windowBackgroundColor

        // Create view controller
        mainViewController = MainViewController()

        // CRITICAL: Set content view controller
        window.contentViewController = mainViewController

        print("📐 Window frame after contentViewController: \(window.frame)")

        // Show the window FIRST, then force frame
        window.center()
        window.makeKeyAndOrderFront(nil)

        print("📐 Window frame after makeKeyAndOrderFront: \(window.frame)")

        // FORCE the frame AFTER the window is visible
        // This is the LAST operation to override any auto-sizing
        let desiredFrame = NSRect(x: window.frame.origin.x, y: window.frame.origin.y, width: 1200, height: 832)
        window.setFrame(desiredFrame, display: true)

        // FORCE contentView to resize to match window
        window.contentView?.frame = NSRect(x: 0, y: 0, width: 1200, height: 800)
        window.contentView?.needsLayout = true
        window.contentView?.layoutSubtreeIfNeeded()

        // Force mainViewController view to match
        mainViewController?.view.frame = NSRect(x: 0, y: 0, width: 1200, height: 800)
        mainViewController?.view.needsLayout = true
        mainViewController?.view.layoutSubtreeIfNeeded()

        // Force complete redisplay
        window.display()
        window.contentView?.display()

        NSApp.activate(ignoringOtherApps: true)

        self.mainWindow = window

        print("✅ Final window frame: \(window.frame)")
        print("✅ Content view frame: \(window.contentView?.frame ?? .zero)")
        print("✅ ViewController view frame: \(mainViewController?.view.frame ?? .zero)")
    }

    @objc private func showAbout() {
        let alert = NSAlert()
        alert.messageText = "OxVCS"
        alert.informativeText = "Logic Pro Version Control\nPowered by Oxen\n\nVersion 1.0.0"
        alert.alertStyle = .informational
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }

    @objc private func showSettings() {
        let settingsWindow = SettingsWindow()
        settingsWindow.show()
    }

    @objc private func showProjectWizard() {
        let wizard = ProjectWizardWindow()
        wizard.show()
    }

    @objc private func refreshProjectList() {
        mainViewController?.refreshProjects()
    }

    @objc private func showMergeHelper() {
        if let selectedProject = mainViewController?.selectedProject {
            let mergeHelper = MergeHelperWindow(project: selectedProject)
            mergeHelper.show()
        } else {
            let alert = NSAlert()
            alert.messageText = "No Project Selected"
            alert.informativeText = "Please select a project first"
            alert.alertStyle = .informational
            alert.addButton(withTitle: "OK")
            alert.runModal()
        }
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
