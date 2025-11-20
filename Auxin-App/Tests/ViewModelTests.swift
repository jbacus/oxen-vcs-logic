import XCTest
@testable import AuxinApp

/// Tests for ViewModels
final class ViewModelTests: XCTestCase {

    // MARK: - ProjectListViewModel Tests

    func testProjectListViewModelInitialization() {
        let viewModel = ProjectListViewModel()

        XCTAssertTrue(viewModel.projects.isEmpty)
        XCTAssertFalse(viewModel.isLoading)
        XCTAssertNil(viewModel.error)
    }

    func testProjectListViewModelAddProject() async {
        let viewModel = ProjectListViewModel()
        let mockClient = MockOxenDaemonXPCClient()
        viewModel.xpcClient = mockClient

        let project = Project(
            id: "new",
            name: "New Project",
            path: "/path/New.logicx",
            type: .logicPro
        )

        await viewModel.addProject(project)

        XCTAssertEqual(viewModel.projects.count, 1)
    }

    func testProjectListViewModelRemoveProject() async {
        let viewModel = ProjectListViewModel()
        let mockClient = MockOxenDaemonXPCClient()
        viewModel.xpcClient = mockClient

        let project = Project(
            id: "to-remove",
            name: "Remove Me",
            path: "/path/Remove.logicx",
            type: .logicPro
        )

        await viewModel.addProject(project)
        await viewModel.removeProject(project)

        XCTAssertTrue(viewModel.projects.isEmpty)
    }

    func testProjectListViewModelRefresh() async {
        let viewModel = ProjectListViewModel()
        let mockClient = MockOxenDaemonXPCClient()
        viewModel.xpcClient = mockClient

        await viewModel.refresh()

        // Should complete without error
        XCTAssertFalse(viewModel.isLoading)
    }

    func testProjectListViewModelErrorHandling() async {
        let viewModel = ProjectListViewModel()
        let mockClient = MockOxenDaemonXPCClient()
        mockClient.shouldFail = true
        viewModel.xpcClient = mockClient

        await viewModel.refresh()

        XCTAssertNotNil(viewModel.error)
    }

    func testProjectListViewModelSelection() {
        let viewModel = ProjectListViewModel()

        let project = Project(
            id: "select",
            name: "Select Me",
            path: "/path",
            type: .logicPro
        )

        viewModel.projects = [project]
        viewModel.selectedProjectId = "select"

        XCTAssertEqual(viewModel.selectedProject?.id, "select")
    }

    func testProjectListViewModelSort() {
        let viewModel = ProjectListViewModel()

        viewModel.projects = [
            Project(id: "2", name: "B Project", path: "/b", type: .logicPro),
            Project(id: "1", name: "A Project", path: "/a", type: .logicPro),
            Project(id: "3", name: "C Project", path: "/c", type: .logicPro)
        ]

        viewModel.sortProjects(by: .name)

        XCTAssertEqual(viewModel.projects[0].name, "A Project")
        XCTAssertEqual(viewModel.projects[1].name, "B Project")
        XCTAssertEqual(viewModel.projects[2].name, "C Project")
    }

    func testProjectListViewModelFilter() {
        let viewModel = ProjectListViewModel()

        viewModel.projects = [
            Project(id: "1", name: "Logic Project", path: "/a.logicx", type: .logicPro),
            Project(id: "2", name: "SketchUp Model", path: "/b.skp", type: .sketchUp)
        ]

        let filtered = viewModel.filteredProjects(type: .logicPro)

        XCTAssertEqual(filtered.count, 1)
        XCTAssertEqual(filtered[0].type, .logicPro)
    }

    // MARK: - ProjectDetailViewModel Tests

    func testProjectDetailViewModelInitialization() {
        let project = Project(
            id: "detail",
            name: "Detail Test",
            path: "/path",
            type: .logicPro
        )

        let viewModel = ProjectDetailViewModel(project: project)

        XCTAssertEqual(viewModel.project.id, "detail")
        XCTAssertFalse(viewModel.isLoading)
    }

    func testProjectDetailViewModelLoadHistory() async {
        let project = Project(
            id: "history",
            name: "History Test",
            path: "/path",
            type: .logicPro
        )

        let viewModel = ProjectDetailViewModel(project: project)
        let mockClient = MockOxenDaemonXPCClient()
        viewModel.xpcClient = mockClient

        await viewModel.loadHistory()

        // Should complete
        XCTAssertFalse(viewModel.isLoading)
    }

    func testProjectDetailViewModelAcquireLock() async {
        let project = Project(
            id: "lock",
            name: "Lock Test",
            path: "/path",
            type: .logicPro
        )

        let viewModel = ProjectDetailViewModel(project: project)
        let mockClient = MockOxenDaemonXPCClient()
        viewModel.xpcClient = mockClient

        let result = await viewModel.acquireLock(timeout: 4)

        XCTAssertTrue(result)
        XCTAssertTrue(viewModel.project.isLockedByMe)
    }

    func testProjectDetailViewModelReleaseLock() async {
        let project = Project(
            id: "release",
            name: "Release Test",
            path: "/path",
            type: .logicPro
        )

        var viewModel = ProjectDetailViewModel(project: project)
        viewModel.project.lockStatus = .lockedByMe
        let mockClient = MockOxenDaemonXPCClient()
        viewModel.xpcClient = mockClient

        let result = await viewModel.releaseLock()

        XCTAssertTrue(result)
        XCTAssertFalse(viewModel.project.isLocked)
    }

    func testProjectDetailViewModelCommit() async {
        let project = Project(
            id: "commit",
            name: "Commit Test",
            path: "/path",
            type: .logicPro
        )

        let viewModel = ProjectDetailViewModel(project: project)
        let mockClient = MockOxenDaemonXPCClient()
        viewModel.xpcClient = mockClient

        let metadata = ProjectMetadata(
            bpm: 120.0,
            sampleRate: 48000,
            key: "C Major",
            tags: nil
        )

        let result = await viewModel.createCommit(
            message: "Test commit",
            metadata: metadata
        )

        XCTAssertTrue(result)
    }

    func testProjectDetailViewModelRestore() async {
        let project = Project(
            id: "restore",
            name: "Restore Test",
            path: "/path",
            type: .logicPro
        )

        let viewModel = ProjectDetailViewModel(project: project)
        let mockClient = MockOxenDaemonXPCClient()
        viewModel.xpcClient = mockClient

        let result = await viewModel.restore(to: "abc123")

        XCTAssertTrue(result)
    }

    func testProjectDetailViewModelStatus() async {
        let project = Project(
            id: "status",
            name: "Status Test",
            path: "/path",
            type: .logicPro
        )

        let viewModel = ProjectDetailViewModel(project: project)
        let mockClient = MockOxenDaemonXPCClient()
        viewModel.xpcClient = mockClient

        await viewModel.refreshStatus()

        XCTAssertNotEqual(viewModel.project.status, .unknown)
    }

    func testProjectDetailViewModelMetadataUpdate() {
        let project = Project(
            id: "meta",
            name: "Metadata Test",
            path: "/path",
            type: .logicPro
        )

        let viewModel = ProjectDetailViewModel(project: project)

        viewModel.updateMetadata(bpm: 140.0, key: "D Minor")

        XCTAssertEqual(viewModel.pendingMetadata?.bpm, 140.0)
        XCTAssertEqual(viewModel.pendingMetadata?.key, "D Minor")
    }

    // MARK: - Observable Tests

    func testViewModelPublishedProperties() {
        let viewModel = ProjectListViewModel()

        // Test that @Published properties trigger updates
        var updateCount = 0
        let cancellable = viewModel.$projects.sink { _ in
            updateCount += 1
        }

        viewModel.projects = [
            Project(id: "1", name: "Test", path: "/path", type: .logicPro)
        ]

        XCTAssertGreaterThan(updateCount, 0)
        cancellable.cancel()
    }

    // MARK: - Concurrent Access Tests

    func testViewModelConcurrentAccess() async {
        let viewModel = ProjectListViewModel()
        let mockClient = MockOxenDaemonXPCClient()
        viewModel.xpcClient = mockClient

        // Multiple concurrent operations
        await withTaskGroup(of: Void.self) { group in
            for i in 0..<10 {
                group.addTask {
                    let project = Project(
                        id: "\(i)",
                        name: "Project \(i)",
                        path: "/path/\(i)",
                        type: .logicPro
                    )
                    await viewModel.addProject(project)
                }
            }
        }

        // Should not crash and handle concurrent access
        XCTAssertEqual(viewModel.projects.count, 10)
    }
}

/// Mock XPC client for testing
class MockOxenDaemonXPCClient: OxenDaemonXPCClientProtocol {
    var shouldFail = false

    func getStatus(for path: String) async throws -> ProjectStatus {
        if shouldFail {
            throw NSError(domain: "MockError", code: 1)
        }
        return .clean
    }

    func getHistory(for path: String, limit: Int) async throws -> [Commit] {
        if shouldFail {
            throw NSError(domain: "MockError", code: 1)
        }
        return [
            Commit(id: "abc123", message: "Test", author: "user", date: Date())
        ]
    }

    func acquireLock(for path: String, timeout: Int) async throws -> Bool {
        if shouldFail {
            throw NSError(domain: "MockError", code: 1)
        }
        return true
    }

    func releaseLock(for path: String) async throws -> Bool {
        if shouldFail {
            throw NSError(domain: "MockError", code: 1)
        }
        return true
    }

    func commit(path: String, message: String, metadata: ProjectMetadata?) async throws -> Bool {
        if shouldFail {
            throw NSError(domain: "MockError", code: 1)
        }
        return true
    }

    func restore(path: String, commitId: String) async throws -> Bool {
        if shouldFail {
            throw NSError(domain: "MockError", code: 1)
        }
        return true
    }
}
