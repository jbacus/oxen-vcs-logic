import AppKit

/// View for managing project locks
class LockManagementView: NSView {
    private let project: Project
    private let statusLabel: NSTextField
    private let acquireButton: NSButton
    private let releaseButton: NSButton
    private let forceBreakButton: NSButton
    private let lockInfoTextView: NSTextView
    private let refreshButton: NSButton

    init(project: Project) {
        self.project = project

        statusLabel = NSTextField(labelWithString: "")
        acquireButton = NSButton(title: "Acquire Lock", target: nil, action: nil)
        releaseButton = NSButton(title: "Release Lock", target: nil, action: nil)
        forceBreakButton = NSButton(title: "Force Break Lock", target: nil, action: nil)
        lockInfoTextView = NSTextView()
        refreshButton = NSButton(title: "Refresh", target: nil, action: nil)

        super.init(frame: .zero)
        setupUI()
        refreshLockStatus()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setupUI() {
        var yPos: CGFloat = 250

        // Title
        let titleLabel = NSTextField(labelWithString: "Lock Management")
        titleLabel.font = NSFont.systemFont(ofSize: 16, weight: .semibold)
        titleLabel.frame = NSRect(x: 20, y: yPos, width: 400, height: 24)
        addSubview(titleLabel)
        yPos -= 40

        // Status
        statusLabel.frame = NSRect(x: 20, y: yPos, width: 400, height: 20)
        statusLabel.font = NSFont.systemFont(ofSize: 13)
        addSubview(statusLabel)
        yPos -= 30

        // Lock info
        lockInfoTextView.frame = NSRect(x: 20, y: yPos - 80, width: 400, height: 80)
        lockInfoTextView.isEditable = false
        lockInfoTextView.font = NSFont.monospacedSystemFont(ofSize: 11, weight: .regular)
        lockInfoTextView.backgroundColor = NSColor.textBackgroundColor
        addSubview(lockInfoTextView)
        yPos -= 100

        // Buttons
        acquireButton.frame = NSRect(x: 20, y: yPos, width: 120, height: 32)
        acquireButton.bezelStyle = .rounded
        acquireButton.target = self
        acquireButton.action = #selector(acquireLock)
        addSubview(acquireButton)

        releaseButton.frame = NSRect(x: 150, y: yPos, width: 120, height: 32)
        releaseButton.bezelStyle = .rounded
        releaseButton.target = self
        releaseButton.action = #selector(releaseLock)
        addSubview(releaseButton)

        forceBreakButton.frame = NSRect(x: 280, y: yPos, width: 140, height: 32)
        forceBreakButton.bezelStyle = .rounded
        forceBreakButton.target = self
        forceBreakButton.action = #selector(forceBreakLock)
        addSubview(forceBreakButton)
        yPos -= 40

        refreshButton.frame = NSRect(x: 20, y: yPos, width: 100, height: 26)
        refreshButton.bezelStyle = .rounded
        refreshButton.target = self
        refreshButton.action = #selector(refreshLockStatus)
        addSubview(refreshButton)
    }

    @objc private func refreshLockStatus() {
        // This would call the XPC service to get lock info
        // For now, placeholder implementation
        statusLabel.stringValue = "Checking lock status..."

        // Simulated lock check (would actually use XPC client)
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) { [weak self] in
            self?.statusLabel.stringValue = "Lock status updated"
            self?.lockInfoTextView.string = "Not locked"
        }
    }

    @objc private func acquireLock() {
        let alert = NSAlert()
        alert.messageText = "Acquire Lock"
        alert.informativeText = "Lock timeout (hours):"
        alert.alertStyle = .informational
        alert.addButton(withTitle: "Acquire")
        alert.addButton(withTitle: "Cancel")

        let input = NSTextField(frame: NSRect(x: 0, y: 0, width: 100, height: 24))
        input.stringValue = "24"
        alert.accessoryView = input

        let response = alert.runModal()
        guard response == .alertFirstButtonReturn else { return }

        guard let timeout = Int(input.stringValue), timeout > 0 else {
            showError("Invalid timeout value")
            return
        }

        // Acquire lock via XPC
        // OxenDaemonXPCClient.shared.acquireLock(...)
        statusLabel.stringValue = "Lock acquired successfully"
        refreshLockStatus()
    }

    @objc private func releaseLock() {
        let alert = NSAlert()
        alert.messageText = "Release Lock"
        alert.informativeText = "Are you sure you want to release the lock for this project?"
        alert.alertStyle = .warning
        alert.addButton(withTitle: "Release")
        alert.addButton(withTitle: "Cancel")

        let response = alert.runModal()
        guard response == .alertFirstButtonReturn else { return }

        // Release lock via XPC
        // OxenDaemonXPCClient.shared.releaseLock(...)
        statusLabel.stringValue = "Lock released"
        refreshLockStatus()
    }

    @objc private func forceBreakLock() {
        let alert = NSAlert()
        alert.messageText = "Force Break Lock"
        alert.informativeText = "WARNING: This will forcibly remove the lock, even if owned by someone else. Only use in emergencies.\n\nAre you sure?"
        alert.alertStyle = .critical
        alert.addButton(withTitle: "Force Break")
        alert.addButton(withTitle: "Cancel")

        let response = alert.runModal()
        guard response == .alertFirstButtonReturn else { return }

        // Force break lock via XPC
        // OxenDaemonXPCClient.shared.forceBreakLock(...)
        statusLabel.stringValue = "Lock forcibly broken"
        refreshLockStatus()
    }

    private func showError(_ message: String) {
        let alert = NSAlert()
        alert.messageText = "Error"
        alert.informativeText = message
        alert.alertStyle = .warning
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }
}
