import AppKit

class ProjectListView: NSView {
    weak var delegate: ProjectListViewDelegate?

    private let tableView = NSTableView()
    private let scrollView = NSScrollView()
    private var projects: [Project] = []

    override init(frame frameRect: NSRect) {
        super.init(frame: frameRect)
        setupUI()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setupUI() {
        // Setup scroll view with FRAME-BASED layout (not Auto Layout)
        // This is critical for NSSplitView subviews
        scrollView.autoresizingMask = [.width, .height]
        scrollView.hasVerticalScroller = true
        scrollView.hasHorizontalScroller = false
        scrollView.borderType = .noBorder
        scrollView.backgroundColor = .controlBackgroundColor
        scrollView.drawsBackground = true

        // Setup table view
        tableView.delegate = self
        tableView.dataSource = self
        tableView.headerView = nil
        tableView.rowSizeStyle = .medium
        tableView.backgroundColor = .controlBackgroundColor

        // Add column
        let column = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("ProjectColumn"))
        column.title = "Projects"
        column.minWidth = 200
        tableView.addTableColumn(column)

        scrollView.documentView = tableView
        scrollView.frame = bounds
        addSubview(scrollView)
    }

    func updateProjects(_ projects: [Project]) {
        self.projects = projects
        tableView.reloadData()
    }
}

extension ProjectListView: NSTableViewDataSource {
    func numberOfRows(in tableView: NSTableView) -> Int {
        return projects.count
    }
}

extension ProjectListView: NSTableViewDelegate {
    func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
        let project = projects[row]

        let cellView = ProjectCellView()
        cellView.configure(with: project)

        return cellView
    }

    func tableView(_ tableView: NSTableView, heightOfRow row: Int) -> CGFloat {
        return 60
    }

    func tableViewSelectionDidChange(_ notification: Notification) {
        let row = tableView.selectedRow
        guard row >= 0, row < projects.count else { return }

        let project = projects[row]
        delegate?.projectListView(self, didSelectProject: project)
    }
}

// MARK: - Project Cell View

class ProjectCellView: NSView {
    private let nameLabel = NSTextField(labelWithString: "")
    private let pathLabel = NSTextField(labelWithString: "")
    private let statusLabel = NSTextField(labelWithString: "")
    private let lockIcon = NSImageView()

    override init(frame frameRect: NSRect) {
        super.init(frame: frameRect)
        setupUI()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setupUI() {
        // Name label
        nameLabel.font = NSFont.systemFont(ofSize: 14, weight: .medium)
        nameLabel.translatesAutoresizingMaskIntoConstraints = false
        addSubview(nameLabel)

        // Path label
        pathLabel.font = NSFont.systemFont(ofSize: 11)
        pathLabel.textColor = .secondaryLabelColor
        pathLabel.lineBreakMode = .byTruncatingMiddle
        pathLabel.translatesAutoresizingMaskIntoConstraints = false
        addSubview(pathLabel)

        // Status label
        statusLabel.font = NSFont.systemFont(ofSize: 10)
        statusLabel.textColor = .tertiaryLabelColor
        statusLabel.translatesAutoresizingMaskIntoConstraints = false
        addSubview(statusLabel)

        // Lock icon
        lockIcon.image = NSImage(systemSymbolName: "lock.fill", accessibilityDescription: "Locked")
        lockIcon.contentTintColor = .systemOrange
        lockIcon.translatesAutoresizingMaskIntoConstraints = false
        lockIcon.isHidden = true
        addSubview(lockIcon)

        NSLayoutConstraint.activate([
            nameLabel.topAnchor.constraint(equalTo: topAnchor, constant: 8),
            nameLabel.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 12),
            nameLabel.trailingAnchor.constraint(equalTo: lockIcon.leadingAnchor, constant: -8),

            lockIcon.centerYAnchor.constraint(equalTo: nameLabel.centerYAnchor),
            lockIcon.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -12),
            lockIcon.widthAnchor.constraint(equalToConstant: 16),
            lockIcon.heightAnchor.constraint(equalToConstant: 16),

            pathLabel.topAnchor.constraint(equalTo: nameLabel.bottomAnchor, constant: 2),
            pathLabel.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 12),
            pathLabel.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -12),

            statusLabel.topAnchor.constraint(equalTo: pathLabel.bottomAnchor, constant: 2),
            statusLabel.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 12),
            statusLabel.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -12)
        ])
    }

    func configure(with project: Project) {
        nameLabel.stringValue = project.displayName
        pathLabel.stringValue = project.path
        lockIcon.isHidden = !project.isLocked

        if project.isLocked, let lockedBy = project.lockedBy {
            statusLabel.stringValue = "Locked by \(lockedBy)"
        } else if let lastCommit = project.lastCommit {
            let formatter = RelativeDateTimeFormatter()
            formatter.unitsStyle = .abbreviated
            let relativeDate = formatter.localizedString(for: lastCommit, relativeTo: Date())
            statusLabel.stringValue = "\(project.commitCount) commits • Last: \(relativeDate)"
        } else {
            statusLabel.stringValue = "No commits yet"
        }
    }
}
