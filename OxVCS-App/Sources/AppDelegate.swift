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
        // Create window with explicit size
        let windowRect = NSRect(x: 0, y: 0, width: 1200, height: 800)
        let window = NSWindow(
            contentRect: windowRect,
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: false
        )
        window.title = "OxVCS - Logic Pro Version Control"

        // Set size constraints
        window.minSize = NSSize(width: 800, height: 600)
        window.maxSize = NSSize(width: 4096, height: 2160)  // Reasonable maximum
        window.contentMinSize = NSSize(width: 800, height: 600)

        // Ensure window has proper background
        window.backgroundColor = .windowBackgroundColor
        window.isOpaque = true

        // Create and set view controller BEFORE autosave
        mainViewController = MainViewController()
        window.contentViewController = mainViewController

        // Force the content view to have the correct size
        mainViewController!.view.frame = NSRect(x: 0, y: 0, width: 1200, height: 800)
        mainViewController!.view.needsLayout = true
        mainViewController!.view.layoutSubtreeIfNeeded()

        // Now set autosave name (after content is set)
        window.setFrameAutosaveName("MainWindow")

        // Ensure window has correct frame (override any bad saved state)
        var currentFrame = window.frame
        if currentFrame.size.width < 800 || currentFrame.size.height < 600 {
            // Reset to default size if saved frame is too small
            currentFrame.size.width = 1200
            currentFrame.size.height = 800
            window.setFrame(currentFrame, display: false)
        }

        // Center and show
        window.center()
        window.makeKeyAndOrderFront(nil)
        window.display()
        NSApp.activate(ignoringOtherApps: true)

        self.mainWindow = window
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
