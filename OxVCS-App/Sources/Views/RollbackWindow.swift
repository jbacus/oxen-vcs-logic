import AppKit

class RollbackWindow: NSObject {
    private let window: NSWindow
    private let viewModel: ProjectDetailViewModel
    private let tableView: NSTableView
    private let progressIndicator: NSProgressIndicator

    init(viewModel: ProjectDetailViewModel) {
        self.viewModel = viewModel

        let contentView = NSView(frame: NSRect(x: 0, y: 0, width: 700, height: 500))

        window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 700, height: 500),
            styleMask: [.titled, .closable, .resizable],
            backing: .buffered,
            defer: false
        )
        window.title = "Rollback Project"
        window.contentView = contentView
        window.center()

        tableView = NSTableView()
        progressIndicator = NSProgressIndicator()

        setupUI(in: contentView)
    }

    private func setupUI(in contentView: NSView) {
        // Title
        let titleLabel = NSTextField(labelWithString: "Select a commit to restore:")
        titleLabel.font = NSFont.systemFont(ofSize: 16, weight: .semibold)
        titleLabel.frame = NSRect(x: 20, y: 450, width: 660, height: 24)
        contentView.addSubview(titleLabel)

        // Warning label
        let warningLabel = NSTextField(labelWithString: "Warning: This will restore your project to the selected state. Current changes will be lost.")
        warningLabel.font = NSFont.systemFont(ofSize: 12)
        warningLabel.textColor = .systemOrange
        warningLabel.frame = NSRect(x: 20, y: 420, width: 660, height: 20)
        contentView.addSubview(warningLabel)

        // Table view with scroll view
        let scrollView = NSScrollView(frame: NSRect(x: 20, y: 80, width: 660, height: 330))
        scrollView.hasVerticalScroller = true
        scrollView.borderType = .bezelBorder

        tableView.delegate = self
        tableView.dataSource = self
        tableView.usesAlternatingRowBackgroundColors = true

        // Add columns
        let hashColumn = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("hash"))
        hashColumn.title = "Commit"
        hashColumn.width = 100
        tableView.addTableColumn(hashColumn)

        let messageColumn = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("message"))
        messageColumn.title = "Message"
        messageColumn.width = 300
        tableView.addTableColumn(messageColumn)

        let dateColumn = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("date"))
        dateColumn.title = "Date"
        dateColumn.width = 180
        tableView.addTableColumn(dateColumn)

        let authorColumn = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("author"))
        authorColumn.title = "Author"
        authorColumn.width = 120
        tableView.addTableColumn(authorColumn)

        scrollView.documentView = tableView
        contentView.addSubview(scrollView)

        // Progress indicator
        progressIndicator.style = .spinning
        progressIndicator.isHidden = true
        progressIndicator.frame = NSRect(x: 334, y: 30, width: 32, height: 32)
        contentView.addSubview(progressIndicator)

        // Buttons
        let cancelButton = NSButton(title: "Cancel", target: self, action: #selector(cancel))
        cancelButton.frame = NSRect(x: 490, y: 20, width: 90, height: 32)
        cancelButton.bezelStyle = .rounded
        contentView.addSubview(cancelButton)

        let rollbackButton = NSButton(title: "Rollback", target: self, action: #selector(rollback))
        rollbackButton.frame = NSRect(x: 590, y: 20, width: 90, height: 32)
        rollbackButton.bezelStyle = .rounded
        rollbackButton.keyEquivalent = "\r"
        contentView.addSubview(rollbackButton)

        // Load commits
        tableView.reloadData()
    }

    func show() {
        window.makeKeyAndOrderFront(nil)
    }

    @objc private func cancel() {
        window.close()
    }

    @objc private func rollback() {
        let row = tableView.selectedRow
        guard row >= 0, row < viewModel.commits.count else {
            showError("Please select a commit to restore")
            return
        }

        let commit = viewModel.commits[row]

        // Show confirmation dialog
        let alert = NSAlert()
        alert.messageText = "Confirm Rollback"
        alert.informativeText = "Are you sure you want to restore the project to commit \(commit.shortHash)?\n\nMessage: \(commit.message)\nDate: \(commit.formattedDate)\n\nThis operation cannot be undone."
        alert.alertStyle = .warning
        alert.addButton(withTitle: "Rollback")
        alert.addButton(withTitle: "Cancel")

        let response = alert.runModal()
        guard response == .alertFirstButtonReturn else { return }

        // Show progress
        progressIndicator.isHidden = false
        progressIndicator.startAnimation(nil)

        // Perform rollback
        viewModel.restoreToCommit(commit) { [weak self] success in
            DispatchQueue.main.async {
                self?.progressIndicator.stopAnimation(nil)
                self?.progressIndicator.isHidden = true

                if success {
                    self?.window.close()
                    self?.showSuccess("Project successfully restored to commit \(commit.shortHash)")
                } else {
                    self?.showError("Failed to restore project. Please check the logs.")
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

extension RollbackWindow: NSTableViewDataSource {
    func numberOfRows(in tableView: NSTableView) -> Int {
        return viewModel.commits.count
    }
}

extension RollbackWindow: NSTableViewDelegate {
    func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
        let commit = viewModel.commits[row]
        let identifier = tableColumn?.identifier.rawValue ?? ""

        let cell = NSTextField(labelWithString: "")
        cell.lineBreakMode = .byTruncatingTail

        switch identifier {
        case "hash":
            cell.stringValue = commit.shortHash
            cell.font = NSFont.monospacedSystemFont(ofSize: 11, weight: .regular)
        case "message":
            cell.stringValue = commit.message
        case "date":
            cell.stringValue = commit.formattedDate
        case "author":
            cell.stringValue = commit.author
        default:
            break
        }

        return cell
    }

    func tableView(_ tableView: NSTableView, heightOfRow row: Int) -> CGFloat {
        return 24
    }
}
