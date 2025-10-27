import AppKit
import Combine

class ProjectDetailView: NSView {
    private let viewModel: ProjectDetailViewModel
    private var cancellables = Set<AnyCancellable>()

    private let headerView = NSView()
    private let projectNameLabel = NSTextField(labelWithString: "")
    private let commitButton = NSButton(title: "Create Milestone Commit", target: nil, action: nil)
    private let rollbackButton = NSButton(title: "Rollback", target: nil, action: nil)
    private let lockButton = NSButton(title: "Manage Locks", target: nil, action: nil)

    private let tableView = NSTableView()
    private let scrollView = NSScrollView()

    init(project: Project) {
        self.viewModel = ProjectDetailViewModel(project: project)
        super.init(frame: .zero)
        setupUI()
        bindViewModel()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setupUI() {
        // Header setup
        headerView.translatesAutoresizingMaskIntoConstraints = false
        addSubview(headerView)

        projectNameLabel.font = NSFont.systemFont(ofSize: 20, weight: .semibold)
        projectNameLabel.stringValue = viewModel.project.displayName
        projectNameLabel.translatesAutoresizingMaskIntoConstraints = false
        headerView.addSubview(projectNameLabel)

        commitButton.bezelStyle = .rounded
        commitButton.target = self
        commitButton.action = #selector(createMilestoneCommit)
        commitButton.translatesAutoresizingMaskIntoConstraints = false
        headerView.addSubview(commitButton)

        rollbackButton.bezelStyle = .rounded
        rollbackButton.target = self
        rollbackButton.action = #selector(showRollbackView)
        rollbackButton.translatesAutoresizingMaskIntoConstraints = false
        headerView.addSubview(rollbackButton)

        lockButton.bezelStyle = .rounded
        lockButton.target = self
        lockButton.action = #selector(showLockManagement)
        lockButton.translatesAutoresizingMaskIntoConstraints = false
        headerView.addSubview(lockButton)

        // Table setup
        scrollView.translatesAutoresizingMaskIntoConstraints = false
        scrollView.hasVerticalScroller = true
        scrollView.borderType = .bezelBorder

        tableView.delegate = self
        tableView.dataSource = self
        tableView.usesAlternatingRowBackgroundColors = true
        tableView.columnAutoresizingStyle = .uniformColumnAutoresizingStyle

        // Add columns
        let hashColumn = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("hash"))
        hashColumn.title = "Commit"
        hashColumn.minWidth = 80
        hashColumn.width = 100
        tableView.addTableColumn(hashColumn)

        let messageColumn = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("message"))
        messageColumn.title = "Message"
        messageColumn.minWidth = 200
        messageColumn.width = 400
        tableView.addTableColumn(messageColumn)

        let dateColumn = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("date"))
        dateColumn.title = "Date"
        dateColumn.minWidth = 150
        dateColumn.width = 180
        tableView.addTableColumn(dateColumn)

        let authorColumn = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("author"))
        authorColumn.title = "Author"
        authorColumn.minWidth = 100
        authorColumn.width = 150
        tableView.addTableColumn(authorColumn)

        scrollView.documentView = tableView
        addSubview(scrollView)

        // Layout constraints
        NSLayoutConstraint.activate([
            headerView.topAnchor.constraint(equalTo: topAnchor, constant: 16),
            headerView.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 16),
            headerView.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -16),
            headerView.heightAnchor.constraint(equalToConstant: 60),

            projectNameLabel.topAnchor.constraint(equalTo: headerView.topAnchor),
            projectNameLabel.leadingAnchor.constraint(equalTo: headerView.leadingAnchor),

            commitButton.topAnchor.constraint(equalTo: projectNameLabel.bottomAnchor, constant: 8),
            commitButton.leadingAnchor.constraint(equalTo: headerView.leadingAnchor),

            rollbackButton.topAnchor.constraint(equalTo: projectNameLabel.bottomAnchor, constant: 8),
            rollbackButton.leadingAnchor.constraint(equalTo: commitButton.trailingAnchor, constant: 8),

            lockButton.topAnchor.constraint(equalTo: projectNameLabel.bottomAnchor, constant: 8),
            lockButton.leadingAnchor.constraint(equalTo: rollbackButton.trailingAnchor, constant: 8),

            scrollView.topAnchor.constraint(equalTo: headerView.bottomAnchor, constant: 16),
            scrollView.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 16),
            scrollView.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -16),
            scrollView.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -16)
        ])
    }

    private func bindViewModel() {
        viewModel.$commits
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                self?.tableView.reloadData()
            }
            .store(in: &cancellables)
    }

    @objc private func createMilestoneCommit() {
        let commitWindow = MilestoneCommitWindow(viewModel: viewModel)
        commitWindow.show()
    }

    @objc private func showRollbackView() {
        let rollbackWindow = RollbackWindow(viewModel: viewModel)
        rollbackWindow.show()
    }

    @objc private func showLockManagement() {
        let lockWindow = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 500, height: 350),
            styleMask: [.titled, .closable],
            backing: .buffered,
            defer: false
        )
        lockWindow.title = "Lock Management"
        lockWindow.center()

        let lockView = LockManagementView(project: viewModel.project)
        lockWindow.contentView = lockView

        lockWindow.makeKeyAndOrderFront(nil)
    }
}

extension ProjectDetailView: NSTableViewDataSource {
    func numberOfRows(in tableView: NSTableView) -> Int {
        return viewModel.commits.count
    }
}

extension ProjectDetailView: NSTableViewDelegate {
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
