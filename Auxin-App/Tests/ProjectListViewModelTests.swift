import XCTest
@testable import Auxin_App

@MainActor
final class ProjectListViewModelTests: XCTestCase {

    var viewModel: ProjectListViewModel!
    var mockXPCClient: MockAuxinXPCClient!

    override func setUp() {
        super.setUp()
        mockXPCClient = MockAuxinXPCClient()
        viewModel = ProjectListViewModel(xpcClient: mockXPCClient)
    }

    override func tearDown() {
        viewModel = nil
        mockXPCClient = nil
        super.tearDown()
    }

    func testCheckDaemonStatus_WhenClientReturnsTrue_ShouldUpdateStatus() async {
        // Given
        mockXPCClient.pingResult = true
        
        // When
        await viewModel.checkDaemonStatus()
        
        // Then
        XCTAssertNotNil(viewModel.daemonStatus)
        XCTAssertTrue(viewModel.daemonStatus!.isRunning)
    }

    func testCheckDaemonStatus_WhenClientReturnsFalse_ShouldUpdateStatus() async {
        // Given
        mockXPCClient.pingResult = false
        
        // When
        await viewModel.checkDaemonStatus()
        
        // Then
        XCTAssertNotNil(viewModel.daemonStatus)
        XCTAssertFalse(viewModel.daemonStatus!.isRunning)
    }

    func testLoadProjects_WhenClientReturnsProjects_ShouldUpdateProjectsList() async {
        // Given
        let projectPath = "/Users/test/Project1.logicx"
        mockXPCClient.monitoredProjects = [projectPath]
        mockXPCClient.commitHistory = [["count": 5, "timestamp": Date()]]
        mockXPCClient.lockInfo = ["isLocked": true, "lockedBy": "John"]

        // When
        await viewModel.loadProjects()

        // Then
        XCTAssertEqual(viewModel.projects.count, 1)
        XCTAssertEqual(viewModel.projects.first?.path, projectPath)
        XCTAssertEqual(viewModel.projects.first?.commitCount, 5)
        XCTAssertTrue(viewModel.projects.first?.isLocked ?? false)
        XCTAssertEqual(viewModel.projects.first?.lockedBy, "John")
        XCTAssertFalse(viewModel.isLoading)
    }
    
    func testLoadProjects_WhenClientReturnsEmpty_ShouldResultInEmptyProjectsList() async {
        // Given
        mockXPCClient.monitoredProjects = []

        // When
        await viewModel.loadProjects()

        // Then
        XCTAssertTrue(viewModel.projects.isEmpty)
        XCTAssertFalse(viewModel.isLoading)
    }

    func testAddProject_WhenSuccessful_ShouldReloadProjects() async {
        // Given
        let projectPath = "/Users/test/NewProject.logicx"
        mockXPCClient.monitoredProjects = [projectPath] // Simulate project being added after registration

        // When
        let result = await viewModel.addProject(path: projectPath)

        // Then
        XCTAssertTrue(result)
        XCTAssertEqual(mockXPCClient.registeredProjects.first, projectPath)
        XCTAssertEqual(viewModel.projects.count, 1)
        XCTAssertEqual(viewModel.projects.first?.path, projectPath)
    }
}
