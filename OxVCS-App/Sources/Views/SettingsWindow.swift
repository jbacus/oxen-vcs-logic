import AppKit

class SettingsWindow {
    private let window: NSWindow
    private let daemonStatusLabel: NSTextField
    private let pauseResumeButton: NSButton

    private var isDaemonPaused = false

    init() {
        let contentView = NSView(frame: NSRect(x: 0, y: 0, width: 500, height: 400))

        window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 500, height: 400),
            styleMask: [.titled, .closable],
            backing: .buffered,
            defer: false
        )
        window.title = "Settings"
        window.contentView = contentView
        window.center()

        daemonStatusLabel = NSTextField(labelWithString: "")
        pauseResumeButton = NSButton(title: "Pause Monitoring", target: nil, action: nil)

        setupUI(in: contentView)
        updateDaemonStatus()
    }

    private func setupUI(in contentView: NSView) {
        var yPos: CGFloat = 350

        // Title
        let titleLabel = NSTextField(labelWithString: "OxVCS Settings")
        titleLabel.font = NSFont.systemFont(ofSize: 16, weight: .semibold)
        titleLabel.frame = NSRect(x: 20, y: yPos, width: 460, height: 24)
        contentView.addSubview(titleLabel)
        yPos -= 50

        // Daemon Section
        let daemonSectionLabel = NSTextField(labelWithString: "Daemon Status")
        daemonSectionLabel.font = NSFont.systemFont(ofSize: 14, weight: .medium)
        daemonSectionLabel.frame = NSRect(x: 20, y: yPos, width: 460, height: 20)
        contentView.addSubview(daemonSectionLabel)
        yPos -= 30

        daemonStatusLabel.frame = NSRect(x: 20, y: yPos, width: 460, height: 20)
        daemonStatusLabel.font = NSFont.systemFont(ofSize: 12)
        contentView.addSubview(daemonStatusLabel)
        yPos -= 35

        pauseResumeButton.frame = NSRect(x: 20, y: yPos, width: 150, height: 32)
        pauseResumeButton.bezelStyle = .rounded
        pauseResumeButton.target = self
        pauseResumeButton.action = #selector(toggleDaemonMonitoring)
        contentView.addSubview(pauseResumeButton)
        yPos -= 60

        // Auto-commit Settings
        let autoCommitLabel = NSTextField(labelWithString: "Auto-Commit Settings")
        autoCommitLabel.font = NSFont.systemFont(ofSize: 14, weight: .medium)
        autoCommitLabel.frame = NSRect(x: 20, y: yPos, width: 460, height: 20)
        contentView.addSubview(autoCommitLabel)
        yPos -= 35

        let debounceLabel = NSTextField(labelWithString: "Debounce Time (seconds):")
        debounceLabel.frame = NSRect(x: 20, y: yPos, width: 200, height: 20)
        contentView.addSubview(debounceLabel)

        let debounceField = NSTextField(frame: NSRect(x: 230, y: yPos, width: 100, height: 22))
        debounceField.placeholderString = "30"
        debounceField.isEnabled = false // TODO: Implement dynamic debounce configuration
        contentView.addSubview(debounceField)
        yPos -= 60

        // Lock Settings
        let lockLabel = NSTextField(labelWithString: "Lock Settings")
        lockLabel.font = NSFont.systemFont(ofSize: 14, weight: .medium)
        lockLabel.frame = NSRect(x: 20, y: yPos, width: 460, height: 20)
        contentView.addSubview(lockLabel)
        yPos -= 35

        let lockTimeoutLabel = NSTextField(labelWithString: "Lock Timeout (hours):")
        lockTimeoutLabel.frame = NSRect(x: 20, y: yPos, width: 200, height: 20)
        contentView.addSubview(lockTimeoutLabel)

        let lockTimeoutField = NSTextField(frame: NSRect(x: 230, y: yPos, width: 100, height: 22))
        lockTimeoutField.placeholderString = "24"
        lockTimeoutField.isEnabled = false // TODO: Implement dynamic timeout configuration
        contentView.addSubview(lockTimeoutField)
        yPos -= 60

        // About Section
        let aboutLabel = NSTextField(labelWithString: "About")
        aboutLabel.font = NSFont.systemFont(ofSize: 14, weight: .medium)
        aboutLabel.frame = NSRect(x: 20, y: yPos, width: 460, height: 20)
        contentView.addSubview(aboutLabel)
        yPos -= 30

        let versionLabel = NSTextField(labelWithString: "Version 1.0.0 (Phase 3)")
        versionLabel.font = NSFont.systemFont(ofSize: 12)
        versionLabel.textColor = .secondaryLabelColor
        versionLabel.frame = NSRect(x: 20, y: yPos, width: 460, height: 20)
        contentView.addSubview(versionLabel)
        yPos -= 20

        let creditsLabel = NSTextField(labelWithString: "Powered by Oxen VCS")
        creditsLabel.font = NSFont.systemFont(ofSize: 12)
        creditsLabel.textColor = .secondaryLabelColor
        creditsLabel.frame = NSRect(x: 20, y: yPos, width: 460, height: 20)
        contentView.addSubview(creditsLabel)
    }

    func show() {
        window.makeKeyAndOrderFront(nil)
    }

    private func updateDaemonStatus() {
        OxenDaemonXPCClient.shared.ping { [weak self] isRunning in
            DispatchQueue.main.async {
                if isRunning {
                    self?.daemonStatusLabel.stringValue = "Daemon is running"
                    self?.daemonStatusLabel.textColor = .systemGreen
                } else {
                    self?.daemonStatusLabel.stringValue = "Daemon is not running"
                    self?.daemonStatusLabel.textColor = .systemRed
                }
            }
        }
    }

    @objc private func toggleDaemonMonitoring() {
        if isDaemonPaused {
            // Resume monitoring
            OxenDaemonXPCClient.shared.resumeMonitoring { [weak self] success in
                DispatchQueue.main.async {
                    if success {
                        self?.isDaemonPaused = false
                        self?.pauseResumeButton.title = "Pause Monitoring"
                        self?.showSuccess("Monitoring resumed")
                    } else {
                        self?.showError("Failed to resume monitoring")
                    }
                }
            }
        } else {
            // Pause monitoring
            OxenDaemonXPCClient.shared.pauseMonitoring { [weak self] success in
                DispatchQueue.main.async {
                    if success {
                        self?.isDaemonPaused = true
                        self?.pauseResumeButton.title = "Resume Monitoring"
                        self?.showSuccess("Monitoring paused")
                    } else {
                        self?.showError("Failed to pause monitoring")
                    }
                }
            }
        }
    }

    private func showError(_ message: String) {
        let alert = NSAlert()
        alert.messageText = "Error"
        alert.informativeText = message
        alert.alertStyle = .warning
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }

    private func showSuccess(_ message: String) {
        let alert = NSAlert()
        alert.messageText = "Success"
        alert.informativeText = message
        alert.alertStyle = .informational
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }
}
