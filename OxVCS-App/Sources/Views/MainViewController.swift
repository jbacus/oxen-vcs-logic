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
        // Create view with explicit frame
        view = NSView(frame: NSRect(x: 0, y: 0, width: 1200, height: 800))
        view.wantsLayer = true
        view.layer?.backgroundColor = NSColor.windowBackgroundColor.cgColor
    }

    // NOTE: This preferredContentSize override is no longer needed since we're not using
    // window.contentViewController (see AppDelegate.setupMainWindow for explanation).
    // We keep it here for historical reference and in case the view controller is used
    // differently in the future.
    override var preferredContentSize: NSSize {
        get {
            return NSSize(width: 1200, height: 800)
        }
        set {
            // Ignore any attempts to change preferred size
            print("⚠️ Attempted to set preferredContentSize to \(newValue) - IGNORED")
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        setupSimpleSplitView()
        bindViewModel()
        startStatusUpdates()
    }

    override func viewDidAppear() {
        super.viewDidAppear()
        // TODO: Re-enable toolbar
        // setupToolbar()
    }

    deinit {
        statusUpdateTimer?.invalidate()
    }

    private func setupSimpleSplitView() {
        // Force view to have correct size BEFORE adding subviews
        view.frame = NSRect(x: 0, y: 0, width: 1200, height: 800)

        // Add status bar at bottom - EXPLICIT SIZE
        statusBar.autoresizingMask = [.width, .maxYMargin]
        statusBar.frame = NSRect(x: 0, y: 0, width: 1200, height: 24)
        view.addSubview(statusBar)

        // Left panel - EXPLICIT SIZE
        let leftPanel = NSView(frame: NSRect(x: 0, y: 24, width: 300, height: 776))
        leftPanel.wantsLayer = true
        leftPanel.layer?.backgroundColor = NSColor.controlBackgroundColor.cgColor
        leftPanel.autoresizingMask = [.height, .maxXMargin]

        let leftLabel = NSTextField(labelWithString: "Project List")
        leftLabel.font = NSFont.systemFont(ofSize: 14)
        leftLabel.frame = NSRect(x: 10, y: 746, width: 280, height: 20)
        leftLabel.autoresizingMask = [.minYMargin]
        leftPanel.addSubview(leftLabel)

        view.addSubview(leftPanel)

        // Right panel - EXPLICIT SIZE
        let rightPanel = NSView(frame: NSRect(x: 300, y: 24, width: 900, height: 776))
        rightPanel.wantsLayer = true
        rightPanel.layer?.backgroundColor = NSColor.textBackgroundColor.cgColor
        rightPanel.autoresizingMask = [.width, .height]

        let rightLabel = NSTextField(labelWithString: "Project Details")
        rightLabel.font = NSFont.systemFont(ofSize: 14)
        rightLabel.frame = NSRect(x: 10, y: 746, width: 500, height: 30)
        rightLabel.autoresizingMask = [.minYMargin]
        rightPanel.addSubview(rightLabel)

        view.addSubview(rightPanel)
    }

    private func setupUI() {
        // Use FRAME-BASED layout for all views (not Auto Layout)
        // Auto Layout + NSSplitView = width collapse bug

        // Ensure view has the correct frame (it should be 1200x800 from loadView)
        let viewWidth = max(view.bounds.width, 1200)
        let viewHeight = max(view.bounds.height, 800)

        // Add status bar at bottom
        statusBar.autoresizingMask = [.width, .maxYMargin]
        statusBar.frame = NSRect(x: 0, y: 0, width: viewWidth, height: 24)
        view.addSubview(statusBar)

        // Setup split view
        splitView.isVertical = true
        splitView.dividerStyle = .thin
        splitView.delegate = self
        splitView.autoresizingMask = [.width, .height]
        splitView.frame = NSRect(x: 0, y: 24, width: viewWidth, height: viewHeight - 24)
        view.addSubview(splitView)

        // Add project list to left side
        projectListView.delegate = self
        let listContainer = NSView()
        listContainer.wantsLayer = true
        listContainer.layer?.backgroundColor = NSColor.controlBackgroundColor.cgColor

        // IMPORTANT: Set autoresizing mask, NOT auto-layout constraints
        // NSSplitView manages its own subview sizing
        listContainer.autoresizingMask = [.height]
        projectListView.autoresizingMask = [.width, .height]

        listContainer.frame = NSRect(x: 0, y: 0, width: 300, height: 800)
        listContainer.addSubview(projectListView)
        projectListView.frame = listContainer.bounds

        splitView.addArrangedSubview(listContainer)

        // Add placeholder for detail view
        let placeholderView = NSView()
        placeholderView.wantsLayer = true
        placeholderView.layer?.backgroundColor = NSColor.textBackgroundColor.cgColor
        placeholderView.autoresizingMask = [.width, .height]
        placeholderView.frame = NSRect(x: 0, y: 0, width: 900, height: 800)

        let label = NSTextField(labelWithString: "Select a project to view details")
        label.font = NSFont.systemFont(ofSize: 16)
        label.textColor = .secondaryLabelColor
        label.autoresizingMask = [.minXMargin, .maxXMargin, .minYMargin, .maxYMargin]
        label.sizeToFit()
        label.frame.origin = NSPoint(
            x: (placeholderView.frame.width - label.frame.width) / 2,
            y: (placeholderView.frame.height - label.frame.height) / 2
        )
        placeholderView.addSubview(label)

        splitView.addArrangedSubview(placeholderView)

        // Set initial split position AFTER adding both subviews
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

// MARK: - Split View Delegate
extension MainViewController: NSSplitViewDelegate {
    func splitView(_ splitView: NSSplitView, constrainMinCoordinate proposedMinimumPosition: CGFloat, ofSubviewAt dividerIndex: Int) -> CGFloat {
        // Minimum width for left panel (project list)
        return 250
    }

    func splitView(_ splitView: NSSplitView, constrainMaxCoordinate proposedMaximumPosition: CGFloat, ofSubviewAt dividerIndex: Int) -> CGFloat {
        // Maximum width for left panel (ensure right panel has at least 500px)
        return splitView.bounds.width - 500
    }

    func splitView(_ splitView: NSSplitView, canCollapseSubview subview: NSView) -> Bool {
        // Don't allow collapsing either panel
        return false
    }
}

// MARK: - Toolbar Item Identifiers
extension NSToolbarItem.Identifier {
    static let addProject = NSToolbarItem.Identifier("AddProject")
    static let refresh = NSToolbarItem.Identifier("Refresh")
}
