import AppKit

class MilestoneCommitWindow {
    private let window: NSWindow
    private let viewModel: ProjectDetailViewModel

    private let messageTextField: NSTextField
    private let bpmTextField: NSTextField
    private let sampleRateTextField: NSTextField
    private let keySignatureTextField: NSTextField
    private let timeSignatureTextField: NSTextField
    private let tagsTextField: NSTextField
    private let cleanupCheckbox: NSButton
    private let progressIndicator: NSProgressIndicator

    init(viewModel: ProjectDetailViewModel) {
        self.viewModel = viewModel

        let contentView = NSView(frame: NSRect(x: 0, y: 0, width: 500, height: 450))

        window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 500, height: 450),
            styleMask: [.titled, .closable],
            backing: .buffered,
            defer: false
        )
        window.title = "Create Milestone Commit"
        window.contentView = contentView
        window.center()

        // UI Elements
        messageTextField = NSTextField(frame: .zero)
        bpmTextField = NSTextField(frame: .zero)
        sampleRateTextField = NSTextField(frame: .zero)
        keySignatureTextField = NSTextField(frame: .zero)
        timeSignatureTextField = NSTextField(frame: .zero)
        tagsTextField = NSTextField(frame: .zero)
        cleanupCheckbox = NSButton(checkboxWithTitle: "Clean up temporary files (Bounces, Freeze Files, Media.localized)", target: nil, action: nil)
        progressIndicator = NSProgressIndicator()

        setupUI(in: contentView)
    }

    private func setupUI(in contentView: NSView) {
        var yPos: CGFloat = 400

        // Title
        let titleLabel = NSTextField(labelWithString: "Create Milestone Commit")
        titleLabel.font = NSFont.systemFont(ofSize: 16, weight: .semibold)
        titleLabel.frame = NSRect(x: 20, y: yPos, width: 460, height: 24)
        contentView.addSubview(titleLabel)
        yPos -= 40

        // Message
        let messageLabel = NSTextField(labelWithString: "Commit Message:")
        messageLabel.frame = NSRect(x: 20, y: yPos, width: 150, height: 20)
        contentView.addSubview(messageLabel)

        messageTextField.frame = NSRect(x: 180, y: yPos, width: 300, height: 22)
        messageTextField.placeholderString = "e.g., Verse 1 completed"
        contentView.addSubview(messageTextField)
        yPos -= 35

        // BPM
        let bpmLabel = NSTextField(labelWithString: "BPM:")
        bpmLabel.frame = NSRect(x: 20, y: yPos, width: 150, height: 20)
        contentView.addSubview(bpmLabel)

        bpmTextField.frame = NSRect(x: 180, y: yPos, width: 100, height: 22)
        bpmTextField.placeholderString = "120"
        contentView.addSubview(bpmTextField)
        yPos -= 35

        // Sample Rate
        let sampleRateLabel = NSTextField(labelWithString: "Sample Rate:")
        sampleRateLabel.frame = NSRect(x: 20, y: yPos, width: 150, height: 20)
        contentView.addSubview(sampleRateLabel)

        sampleRateTextField.frame = NSRect(x: 180, y: yPos, width: 100, height: 22)
        sampleRateTextField.placeholderString = "44100"
        contentView.addSubview(sampleRateTextField)
        yPos -= 35

        // Key Signature
        let keyLabel = NSTextField(labelWithString: "Key Signature:")
        keyLabel.frame = NSRect(x: 20, y: yPos, width: 150, height: 20)
        contentView.addSubview(keyLabel)

        keySignatureTextField.frame = NSRect(x: 180, y: yPos, width: 100, height: 22)
        keySignatureTextField.placeholderString = "C Major"
        contentView.addSubview(keySignatureTextField)
        yPos -= 35

        // Time Signature
        let timeLabel = NSTextField(labelWithString: "Time Signature:")
        timeLabel.frame = NSRect(x: 20, y: yPos, width: 150, height: 20)
        contentView.addSubview(timeLabel)

        timeSignatureTextField.frame = NSRect(x: 180, y: yPos, width: 100, height: 22)
        timeSignatureTextField.placeholderString = "4/4"
        contentView.addSubview(timeSignatureTextField)
        yPos -= 35

        // Tags
        let tagsLabel = NSTextField(labelWithString: "Tags (comma-separated):")
        tagsLabel.frame = NSRect(x: 20, y: yPos, width: 150, height: 20)
        contentView.addSubview(tagsLabel)

        tagsTextField.frame = NSRect(x: 180, y: yPos, width: 300, height: 22)
        tagsTextField.placeholderString = "verse, vocals, production"
        contentView.addSubview(tagsTextField)
        yPos -= 35

        // Cleanup checkbox
        cleanupCheckbox.frame = NSRect(x: 20, y: yPos, width: 460, height: 20)
        cleanupCheckbox.state = .on
        contentView.addSubview(cleanupCheckbox)
        yPos -= 50

        // Progress indicator
        progressIndicator.style = .spinning
        progressIndicator.isHidden = true
        progressIndicator.frame = NSRect(x: 230, y: yPos + 8, width: 32, height: 32)
        contentView.addSubview(progressIndicator)

        // Buttons
        let cancelButton = NSButton(title: "Cancel", target: self, action: #selector(cancel))
        cancelButton.frame = NSRect(x: 300, y: yPos, width: 90, height: 32)
        cancelButton.bezelStyle = .rounded
        contentView.addSubview(cancelButton)

        let commitButton = NSButton(title: "Commit", target: self, action: #selector(commit))
        commitButton.frame = NSRect(x: 400, y: yPos, width: 90, height: 32)
        commitButton.bezelStyle = .rounded
        commitButton.keyEquivalent = "\r"
        contentView.addSubview(commitButton)
    }

    func show() {
        window.makeKeyAndOrderFront(nil)
    }

    @objc private func cancel() {
        window.close()
    }

    @objc private func commit() {
        // Validate message
        guard !messageTextField.stringValue.isEmpty else {
            showError("Please enter a commit message")
            return
        }

        // Perform pre-flight cleanup if requested
        if cleanupCheckbox.state == .on {
            performCleanup()
        }

        // Collect metadata
        var metadata = CommitMetadata(
            bpm: nil,
            sampleRate: nil,
            keySignature: nil,
            timeSignature: nil,
            tags: nil
        )

        if let bpm = Double(bpmTextField.stringValue) {
            metadata = CommitMetadata(
                bpm: bpm,
                sampleRate: metadata.sampleRate,
                keySignature: metadata.keySignature,
                timeSignature: metadata.timeSignature,
                tags: metadata.tags
            )
        }

        if let sampleRate = Int(sampleRateTextField.stringValue) {
            metadata = CommitMetadata(
                bpm: metadata.bpm,
                sampleRate: sampleRate,
                keySignature: metadata.keySignature,
                timeSignature: metadata.timeSignature,
                tags: metadata.tags
            )
        }

        if !keySignatureTextField.stringValue.isEmpty {
            metadata = CommitMetadata(
                bpm: metadata.bpm,
                sampleRate: metadata.sampleRate,
                keySignature: keySignatureTextField.stringValue,
                timeSignature: metadata.timeSignature,
                tags: metadata.tags
            )
        }

        if !timeSignatureTextField.stringValue.isEmpty {
            metadata = CommitMetadata(
                bpm: metadata.bpm,
                sampleRate: metadata.sampleRate,
                keySignature: metadata.keySignature,
                timeSignature: timeSignatureTextField.stringValue,
                tags: metadata.tags
            )
        }

        if !tagsTextField.stringValue.isEmpty {
            let tags = tagsTextField.stringValue.split(separator: ",").map { $0.trimmingCharacters(in: .whitespaces) }
            metadata = CommitMetadata(
                bpm: metadata.bpm,
                sampleRate: metadata.sampleRate,
                keySignature: metadata.keySignature,
                timeSignature: metadata.timeSignature,
                tags: tags
            )
        }

        // Show progress
        progressIndicator.isHidden = false
        progressIndicator.startAnimation(nil)

        // Create commit
        viewModel.createMilestoneCommit(message: messageTextField.stringValue, metadata: metadata) { [weak self] success in
            DispatchQueue.main.async {
                self?.progressIndicator.stopAnimation(nil)
                self?.progressIndicator.isHidden = true

                if success {
                    self?.window.close()
                    self?.showSuccess("Milestone commit created successfully")
                } else {
                    self?.showError("Failed to create milestone commit")
                }
            }
        }
    }

    private func performCleanup() {
        let projectPath = viewModel.project.path
        let foldersToClean = ["Bounces", "Freeze Files", "Media.localized"]

        for folder in foldersToClean {
            let folderPath = (projectPath as NSString).appendingPathComponent(folder)
            try? FileManager.default.removeItem(atPath: folderPath)
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
