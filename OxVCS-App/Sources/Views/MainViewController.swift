import AppKit
import Combine

class MainViewController: NSViewController {
    private var viewModel = ProjectListViewModel()
    private var cancellables = Set<AnyCancellable>()

    private let splitView = NSSplitView()
    private let projectListView = ProjectListView()
    private var projectDetailView: ProjectDetailView?
    private(set) var selectedProject: Project?

    // Toolbar and status bar
    private let toolbar = NSToolbar(identifier: "MainToolbar")
    private let statusBar = StatusBarView()
    private var statusUpdateTimer: Timer?

    override func loadView() {
        view = NSView(frame: NSRect(x: 0, y: 0, width: 1200, height: 800))
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        setupUI()
        setupToolbar()
        bindViewModel()
        startStatusUpdates()
    }

    deinit {
        statusUpdateTimer?.invalidate()
    }

    private func setupUI() {
        // Add status bar at bottom
        statusBar.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(statusBar)

        // Setup split view
        splitView.isVertical = true
        splitView.dividerStyle = .thin
        splitView.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(splitView)

        NSLayoutConstraint.activate([
            // Status bar at bottom
            statusBar.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            statusBar.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            statusBar.bottomAnchor.constraint(equalTo: view.bottomAnchor),
            statusBar.heightAnchor.constraint(equalToConstant: 24),

            // Split view fills remaining space
            splitView.topAnchor.constraint(equalTo: view.topAnchor),
            splitView.bottomAnchor.constraint(equalTo: statusBar.topAnchor),
            splitView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            splitView.trailingAnchor.constraint(equalTo: view.trailingAnchor)
        ])

        // Add project list to left side
        projectListView.delegate = self
        let listContainer = NSView()
        listContainer.addSubview(projectListView)
        projectListView.translatesAutoresizingMaskIntoConstraints = false
        NSLayoutConstraint.activate([
            projectListView.topAnchor.constraint(equalTo: listContainer.topAnchor),
            projectListView.bottomAnchor.constraint(equalTo: listContainer.bottomAnchor),
            projectListView.leadingAnchor.constraint(equalTo: listContainer.leadingAnchor),
            projectListView.trailingAnchor.constraint(equalTo: listContainer.trailingAnchor)
        ])

        splitView.addArrangedSubview(listContainer)

        // Add placeholder for detail view
        let placeholderView = NSView()
        let label = NSTextField(labelWithString: "Select a project to view details")
        label.font = NSFont.systemFont(ofSize: 16)
        label.textColor = .secondaryLabelColor
        label.translatesAutoresizingMaskIntoConstraints = false
        placeholderView.addSubview(label)
        NSLayoutConstraint.activate([
            label.centerXAnchor.constraint(equalTo: placeholderView.centerXAnchor),
            label.centerYAnchor.constraint(equalTo: placeholderView.centerYAnchor)
        ])

        splitView.addArrangedSubview(placeholderView)

        // Set initial split position (30% / 70%)
        splitView.setPosition(300, ofDividerAt: 0)
    }

    private func bindViewModel() {
        viewModel.$projects
            .receive(on: DispatchQueue.main)
            .sink { [weak self] projects in
                self?.projectListView.updateProjects(projects)
            }
            .store(in: &cancellables)

        viewModel.$errorMessage
            .compactMap { $0 }
            .receive(on: DispatchQueue.main)
            .sink { [weak self] error in
                self?.showError(error)
            }
            .store(in: &cancellables)
    }

    private func showError(_ message: String) {
        let alert = NSAlert()
        alert.messageText = "Error"
        alert.informativeText = message
        alert.alertStyle = .warning
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }

    func refreshProjects() {
        viewModel.loadProjects()
    }

    private func setupToolbar() {
        guard let window = view.window else { return }

        toolbar.delegate = self
        toolbar.displayMode = .iconAndLabel
        toolbar.allowsUserCustomization = true
        toolbar.autosavesConfiguration = true

        window.toolbar = toolbar
    }

    private func startStatusUpdates() {
        updateStatus()
        statusUpdateTimer = Timer.scheduledTimer(withTimeInterval: 5.0, repeats: true) { [weak self] _ in
            self?.updateStatus()
        }
    }

    private func updateStatus() {
        OxenDaemonXPCClient.shared.ping { [weak self] isRunning in
            DispatchQueue.main.async {
                if isRunning {
                    self?.statusBar.updateStatus(message: "Daemon: Running", color: .systemGreen)
                } else {
                    self?.statusBar.updateStatus(message: "Daemon: Not Running", color: .systemRed)
                }

                // Update project count
                let projectCount = self?.viewModel.projects.count ?? 0
                self?.statusBar.updateProjectCount(projectCount)
            }
        }
    }

    @objc private func addProject() {
        let wizard = ProjectWizardWindow()
        wizard.show()
    }

    @objc private func refreshAll() {
        refreshProjects()
        updateStatus()
    }
}

extension MainViewController: NSToolbarDelegate {
    func toolbar(_ toolbar: NSToolbar, itemForItemIdentifier itemIdentifier: NSToolbarItem.Identifier, willBeInsertedIntoToolbar flag: Bool) -> NSToolbarItem? {
        switch itemIdentifier {
        case .addProject:
            let item = NSToolbarItem(itemIdentifier: itemIdentifier)
            item.label = "Add Project"
            item.paletteLabel = "Add Project"
            item.toolTip = "Initialize a new Logic Pro project"
            item.image = NSImage(systemSymbolName: "plus.circle", accessibilityDescription: "Add")
            item.target = self
            item.action = #selector(addProject)
            return item

        case .refresh:
            let item = NSToolbarItem(itemIdentifier: itemIdentifier)
            item.label = "Refresh"
            item.paletteLabel = "Refresh"
            item.toolTip = "Refresh project list"
            item.image = NSImage(systemSymbolName: "arrow.clockwise", accessibilityDescription: "Refresh")
            item.target = self
            item.action = #selector(refreshAll)
            return item

        default:
            return nil
        }
    }

    func toolbarDefaultItemIdentifiers(_ toolbar: NSToolbar) -> [NSToolbarItem.Identifier] {
        return [.addProject, .flexibleSpace, .refresh]
    }

    func toolbarAllowedItemIdentifiers(_ toolbar: NSToolbar) -> [NSToolbarItem.Identifier] {
        return [.addProject, .refresh, .flexibleSpace, .space]
    }
}

extension MainViewController: ProjectListViewDelegate {
    func projectListView(_ view: ProjectListView, didSelectProject project: Project) {
        // Store selected project
        self.selectedProject = project

        // Remove existing detail view
        if splitView.arrangedSubviews.count > 1 {
            splitView.arrangedSubviews[1].removeFromSuperview()
        }

        // Add new detail view
        let detailView = ProjectDetailView(project: project)
        splitView.addArrangedSubview(detailView)
        projectDetailView = detailView

        // Restore split position
        splitView.setPosition(300, ofDividerAt: 0)
    }
}

protocol ProjectListViewDelegate: AnyObject {
    func projectListView(_ view: ProjectListView, didSelectProject project: Project)
}

// MARK: - Toolbar Item Identifiers
extension NSToolbarItem.Identifier {
    static let addProject = NSToolbarItem.Identifier("AddProject")
    static let refresh = NSToolbarItem.Identifier("Refresh")
}
