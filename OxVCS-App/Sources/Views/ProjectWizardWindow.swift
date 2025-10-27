import AppKit

class ProjectWizardWindow {
    private let window: NSWindow
    private let pathTextField: NSTextField
    private let statusLabel: NSTextField
    private let progressIndicator: NSProgressIndicator

    init() {
        let contentView = NSView(frame: NSRect(x: 0, y: 0, width: 600, height: 350))

        window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 600, height: 350),
            styleMask: [.titled, .closable],
            backing: .buffered,
            defer: false
        )
        window.title = "Initialize New Project"
        window.contentView = contentView
        window.center()

        pathTextField = NSTextField(frame: .zero)
        statusLabel = NSTextField(labelWithString: "")
        progressIndicator = NSProgressIndicator()

        setupUI(in: contentView)
    }

    private func setupUI(in contentView: NSView) {
        var yPos: CGFloat = 300

        // Title
        let titleLabel = NSTextField(labelWithString: "Initialize Logic Pro Project")
        titleLabel.font = NSFont.systemFont(ofSize: 18, weight: .semibold)
        titleLabel.frame = NSRect(x: 20, y: yPos, width: 560, height: 28)
        contentView.addSubview(titleLabel)
        yPos -= 50

        // Instructions
        let instructionsLabel = NSTextField(wrappingLabelWithString: "Select a Logic Pro project (.logicx) to initialize with version control. This will create a new Oxen repository and start monitoring the project for changes.")
        instructionsLabel.font = NSFont.systemFont(ofSize: 13)
        instructionsLabel.frame = NSRect(x: 20, y: yPos - 40, width: 560, height: 60)
        contentView.addSubview(instructionsLabel)
        yPos -= 90

        // Project path selection
        let pathLabel = NSTextField(labelWithString: "Project Path:")
        pathLabel.frame = NSRect(x: 20, y: yPos, width: 100, height: 20)
        contentView.addSubview(pathLabel)

        pathTextField.frame = NSRect(x: 130, y: yPos, width: 360, height: 22)
        pathTextField.placeholderString = "/path/to/project.logicx"
        contentView.addSubview(pathTextField)

        let browseButton = NSButton(title: "Browse...", target: self, action: #selector(browse))
        browseButton.frame = NSRect(x: 500, y: yPos - 2, width: 80, height: 26)
        browseButton.bezelStyle = .rounded
        contentView.addSubview(browseButton)
        yPos -= 50

        // Status label
        statusLabel.frame = NSRect(x: 20, y: yPos, width: 560, height: 40)
        statusLabel.font = NSFont.systemFont(ofSize: 12)
        statusLabel.textColor = .secondaryLabelColor
        statusLabel.lineBreakMode = .byWordWrapping
        statusLabel.maximumNumberOfLines = 2
        contentView.addSubview(statusLabel)
        yPos -= 60

        // Progress indicator
        progressIndicator.style = .spinning
        progressIndicator.isHidden = true
        progressIndicator.frame = NSRect(x: 284, y: yPos + 8, width: 32, height: 32)
        contentView.addSubview(progressIndicator)

        // Buttons
        let cancelButton = NSButton(title: "Cancel", target: self, action: #selector(cancel))
        cancelButton.frame = NSRect(x: 390, y: yPos, width: 90, height: 32)
        cancelButton.bezelStyle = .rounded
        contentView.addSubview(cancelButton)

        let initButton = NSButton(title: "Initialize", target: self, action: #selector(initialize))
        initButton.frame = NSRect(x: 490, y: yPos, width: 90, height: 32)
        initButton.bezelStyle = .rounded
        initButton.keyEquivalent = "\r"
        contentView.addSubview(initButton)
    }

    func show() {
        window.makeKeyAndOrderFront(nil)
    }

    @objc private func browse() {
        let openPanel = NSOpenPanel()
        openPanel.canChooseFiles = true
        openPanel.canChooseDirectories = false
        openPanel.allowsMultipleSelection = false
        openPanel.allowedContentTypes = []
        openPanel.allowsOtherFileTypes = true
        openPanel.prompt = "Select Logic Pro Project"

        openPanel.begin { [weak self] response in
            guard response == .OK, let url = openPanel.url else { return }

            // Validate .logicx extension
            if url.pathExtension == "logicx" {
                self?.pathTextField.stringValue = url.path
                self?.statusLabel.stringValue = "Ready to initialize: \(url.lastPathComponent)"
                self?.statusLabel.textColor = .systemGreen
            } else {
                self?.showError("Please select a valid Logic Pro project (.logicx)")
            }
        }
    }

    @objc private func cancel() {
        window.close()
    }

    @objc private func initialize() {
        let path = pathTextField.stringValue

        // Validate path
        guard !path.isEmpty else {
            showError("Please select a project path")
            return
        }

        guard path.hasSuffix(".logicx") else {
            showError("Selected file must be a Logic Pro project (.logicx)")
            return
        }

        guard FileManager.default.fileExists(atPath: path) else {
            showError("Selected project does not exist")
            return
        }

        // Show progress
        statusLabel.stringValue = "Initializing Oxen repository..."
        statusLabel.textColor = .secondaryLabelColor
        progressIndicator.isHidden = false
        progressIndicator.startAnimation(nil)

        // Initialize project via XPC (this calls the CLI to run 'oxenvcs-cli init')
        OxenDaemonXPCClient.shared.initializeProject(path: path) { [weak self] success, error in
            DispatchQueue.main.async {
                self?.progressIndicator.stopAnimation(nil)
                self?.progressIndicator.isHidden = true

                if success {
                    self?.statusLabel.stringValue = "Project initialized successfully!"
                    self?.statusLabel.textColor = .systemGreen

                    // Close window after a delay
                    DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) {
                        self?.window.close()
                    }

                    self?.showSuccess("Project initialized successfully!\n\nThe project is now being monitored for changes. Automatic commits will be created after 30 seconds of inactivity.")
                } else {
                    self?.statusLabel.stringValue = "Failed to initialize project"
                    self?.statusLabel.textColor = .systemRed

                    let errorMessage = error ?? "Unknown error occurred"
                    self?.showError("Failed to initialize project:\n\n\(errorMessage)\n\nMake sure the oxenvcs-cli tool is installed at /usr/local/bin/oxenvcs-cli")
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
