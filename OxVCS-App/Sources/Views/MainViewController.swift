import AppKit
import Combine

class MainViewController: NSViewController {
    private var viewModel = ProjectListViewModel()
    private var cancellables = Set<AnyCancellable>()

    private let splitView = NSSplitView()
    private let projectListView = ProjectListView()
    private var projectDetailView: ProjectDetailView?

    override func loadView() {
        view = NSView(frame: NSRect(x: 0, y: 0, width: 1200, height: 800))
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        setupUI()
        bindViewModel()
    }

    private func setupUI() {
        splitView.isVertical = true
        splitView.dividerStyle = .thin
        splitView.translatesAutoresizingMaskIntoConstraints = false

        view.addSubview(splitView)

        NSLayoutConstraint.activate([
            splitView.topAnchor.constraint(equalTo: view.topAnchor),
            splitView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
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
}

extension MainViewController: ProjectListViewDelegate {
    func projectListView(_ view: ProjectListView, didSelectProject project: Project) {
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
