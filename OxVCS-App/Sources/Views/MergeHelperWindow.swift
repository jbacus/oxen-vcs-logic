import AppKit

/// Helper window for FCP XML merge workflow
class MergeHelperWindow {
    private let window: NSWindow
    private let project: Project

    init(project: Project) {
        self.project = project

        let contentView = NSView(frame: NSRect(x: 0, y: 0, width: 700, height: 600))

        window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 700, height: 600),
            styleMask: [.titled, .closable, .resizable],
            backing: .buffered,
            defer: false
        )
        window.title = "Manual Merge Helper"
        window.contentView = contentView
        window.center()

        setupUI(in: contentView)
    }

    private func setupUI(in contentView: NSView) {
        var yPos: CGFloat = 550

        // Title
        let titleLabel = NSTextField(labelWithString: "FCP XML Merge Workflow")
        titleLabel.font = NSFont.systemFont(ofSize: 18, weight: .semibold)
        titleLabel.frame = NSRect(x: 20, y: yPos, width: 660, height: 28)
        contentView.addSubview(titleLabel)
        yPos -= 40

        // Instructions
        let instructions = NSTextView(frame: NSRect(x: 20, y: yPos - 200, width: 660, height: 200))
        instructions.isEditable = false
        instructions.font = NSFont.systemFont(ofSize: 13)
        instructions.string = """
        Manual merge is required when combining changes from divergent branches.

        This process uses Logic Pro's FCP XML export/import feature to reconcile changes:

        1. Export both versions to FCP XML format
        2. Manually compare and reconcile the XML files
        3. Import the merged XML back into Logic Pro
        4. Verify and commit the result

        See docs/MERGE_PROTOCOL.md for detailed instructions.
        """
        contentView.addSubview(instructions)
        yPos -= 220

        // Step 1: Export Current Version
        addSectionHeader("Step 1: Export Current Version", at: &yPos, in: contentView)
        let exportCurrentButton = NSButton(title: "Open Project in Logic Pro", target: self, action: #selector(openProject))
        exportCurrentButton.frame = NSRect(x: 20, y: yPos, width: 200, height: 32)
        exportCurrentButton.bezelStyle = .rounded
        contentView.addSubview(exportCurrentButton)

        let exportInstructions = NSTextField(wrappingLabelWithString: "Then: File → Export → Project to FCP XML")
        exportInstructions.font = NSFont.systemFont(ofSize: 12)
        exportInstructions.textColor = .secondaryLabelColor
        exportInstructions.frame = NSRect(x: 230, y: yPos + 6, width: 450, height: 20)
        contentView.addSubview(exportInstructions)
        yPos -= 50

        // Step 2: Checkout Other Branch
        addSectionHeader("Step 2: Checkout Other Branch & Export", at: &yPos, in: contentView)
        let branchField = NSTextField(frame: NSRect(x: 20, y: yPos, width: 300, height: 22))
        branchField.placeholderString = "Enter branch name (e.g., feature-branch)"
        contentView.addSubview(branchField)

        let checkoutButton = NSButton(title: "Checkout Branch", target: self, action: #selector(checkoutBranch))
        checkoutButton.frame = NSRect(x: 330, y: yPos - 2, width: 140, height: 26)
        checkoutButton.bezelStyle = .rounded
        contentView.addSubview(checkoutButton)
        yPos -= 40

        // Step 3: Compare & Reconcile
        addSectionHeader("Step 3: Compare & Reconcile XML Files", at: &yPos, in: contentView)
        let diffButton = NSButton(title: "Open Diff Tool", target: self, action: #selector(openDiffTool))
        diffButton.frame = NSRect(x: 20, y: yPos, width: 140, height: 32)
        diffButton.bezelStyle = .rounded
        contentView.addSubview(diffButton)

        let diffInstructions = NSTextField(wrappingLabelWithString: "Compare the two XML files and create a reconciled version")
        diffInstructions.font = NSFont.systemFont(ofSize: 12)
        diffInstructions.textColor = .secondaryLabelColor
        diffInstructions.frame = NSRect(x: 170, y: yPos + 6, width: 510, height: 20)
        contentView.addSubview(diffInstructions)
        yPos -= 50

        // Step 4: Import Merged Version
        addSectionHeader("Step 4: Import Merged Version", at: &yPos, in: contentView)
        let importInstructions = NSTextField(wrappingLabelWithString: "In Logic Pro: File → Import → FCP XML (select your reconciled XML)")
        importInstructions.font = NSFont.systemFont(ofSize: 12)
        importInstructions.textColor = .secondaryLabelColor
        importInstructions.frame = NSRect(x: 20, y: yPos, width: 660, height: 20)
        contentView.addSubview(importInstructions)
        yPos -= 50

        // Step 5: Commit Result
        addSectionHeader("Step 5: Verify & Commit", at: &yPos, in: contentView)
        let commitButton = NSButton(title: "Create Merge Commit", target: self, action: #selector(createMergeCommit))
        commitButton.frame = NSRect(x: 20, y: yPos, width: 180, height: 32)
        commitButton.bezelStyle = .rounded
        contentView.addSubview(commitButton)
        yPos -= 60

        // Documentation link
        let docsButton = NSButton(title: "View Full Documentation", target: self, action: #selector(openDocumentation))
        docsButton.frame = NSRect(x: 20, y: yPos, width: 200, height: 28)
        docsButton.bezelStyle = .rounded
        contentView.addSubview(docsButton)

        // Close button
        let closeButton = NSButton(title: "Close", target: self, action: #selector(close))
        closeButton.frame = NSRect(x: 590, y: yPos, width: 90, height: 28)
        closeButton.bezelStyle = .rounded
        contentView.addSubview(closeButton)
    }

    private func addSectionHeader(_ title: String, at yPos: inout CGFloat, in contentView: NSView) {
        let label = NSTextField(labelWithString: title)
        label.font = NSFont.systemFont(ofSize: 14, weight: .medium)
        label.frame = NSRect(x: 20, y: yPos, width: 660, height: 20)
        contentView.addSubview(label)
        yPos -= 30
    }

    func show() {
        window.makeKeyAndOrderFront(nil)
    }

    @objc private func openProject() {
        // Open the project in Logic Pro
        NSWorkspace.shared.open(URL(fileURLWithPath: project.path))
    }

    @objc private func checkoutBranch() {
        // Show branch selection dialog
        let alert = NSAlert()
        alert.messageText = "Checkout Branch"
        alert.informativeText = "Enter the branch name to checkout:"
        alert.alertStyle = .informational
        alert.addButton(withTitle: "Checkout")
        alert.addButton(withTitle: "Cancel")

        let input = NSTextField(frame: NSRect(x: 0, y: 0, width: 300, height: 24))
        input.placeholderString = "feature-branch"
        alert.accessoryView = input

        let response = alert.runModal()
        guard response == .alertFirstButtonReturn else { return }

        let branchName = input.stringValue
        guard !branchName.isEmpty else { return }

        // Execute checkout via CLI (would need to implement this)
        showInfo("Please run: oxenvcs-cli checkout \(branchName) --project \(project.path)\n\nThen export the project to FCP XML from Logic Pro.")
    }

    @objc private func openDiffTool() {
        let openPanel = NSOpenPanel()
        openPanel.title = "Select First XML File"
        openPanel.allowedContentTypes = []
        openPanel.allowsOtherFileTypes = true
        openPanel.canChooseFiles = true
        openPanel.canChooseDirectories = false
        openPanel.prompt = "Select"

        openPanel.begin { [weak self] response in
            guard response == .OK, let firstFile = openPanel.url else { return }

            let secondPanel = NSOpenPanel()
            secondPanel.title = "Select Second XML File"
            secondPanel.allowedContentTypes = []
            secondPanel.allowsOtherFileTypes = true
            secondPanel.canChooseFiles = true
            secondPanel.canChooseDirectories = false
            secondPanel.prompt = "Compare"

            secondPanel.begin { response in
                guard response == .OK, let secondFile = secondPanel.url else { return }

                // Open both files in diff tool
                let task = Process()
                task.launchPath = "/usr/bin/opendiff"
                task.arguments = [firstFile.path, secondFile.path]
                try? task.run()
            }
        }
    }

    @objc private func createMergeCommit() {
        let commitWindow = MilestoneCommitWindow(
            viewModel: ProjectDetailViewModel(project: project)
        )
        commitWindow.show()
        window.close()
    }

    @objc private func openDocumentation() {
        let docsPath = "/home/user/oxen-vcs-logic/docs/MERGE_PROTOCOL.md"
        NSWorkspace.shared.open(URL(fileURLWithPath: docsPath))
    }

    @objc private func close() {
        window.close()
    }

    private func showInfo(_ message: String) {
        let alert = NSAlert()
        alert.messageText = "Information"
        alert.informativeText = message
        alert.alertStyle = .informational
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }
}
