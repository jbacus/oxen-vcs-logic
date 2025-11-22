import XCTest
@testable import Auxin_App

@MainActor
final class ProjectDetailViewModelTests: XCTestCase {

    var viewModel: ProjectDetailViewModel!
    var mockXPCClient: MockAuxinXPCClient!
    var testProject: Project!

    override func setUp() {
        super.setUp()
        mockXPCClient = MockAuxinXPCClient()
        testProject = Project(
            path: "/Users/test/TestProject.logicx",
            projectType: .logicPro,
            lastCommit: nil,
            commitCount: 0,
            isLocked: false,
            lockedBy: nil
        )
        viewModel = ProjectDetailViewModel(project: testProject, xpcClient: mockXPCClient)
    }

    override func tearDown() {
        viewModel = nil
        mockXPCClient = nil
        testProject = nil
        super.tearDown()
    }

    // MARK: - Load Commit History Tests

    func testLoadCommitHistory_WhenSuccessful_ShouldPopulateCommits() async {
        // Given
        let commit1 = [
            "id": "abc123",
            "message": "First commit",
            "timestamp": Date(),
            "author": "Test User"
        ] as [String: Any]
        let commit2 = [
            "id": "def456",
            "message": "Second commit",
            "timestamp": Date(),
            "author": "Test User"
        ] as [String: Any]
        mockXPCClient.commitHistory = [commit1, commit2]

        // When
        await viewModel.loadCommitHistory()

        // Then
        XCTAssertEqual(viewModel.commits.count, 2)
        XCTAssertEqual(viewModel.commits[0].id, "abc123")
        XCTAssertEqual(viewModel.commits[0].message, "First commit")
        XCTAssertEqual(viewModel.commits[1].id, "def456")
        XCTAssertEqual(viewModel.commits[1].message, "Second commit")
        XCTAssertFalse(viewModel.isLoading)
        XCTAssertNil(viewModel.errorMessage)
    }

    func testLoadCommitHistory_WithMetadata_ShouldParseMetadataCorrectly() async {
        // Given
        let commit = [
            "id": "abc123",
            "message": "Commit with metadata",
            "timestamp": Date(),
            "author": "Test User",
            "metadata": [
                "bpm": 120.0,
                "sample_rate": 44100,
                "key_signature": "C Major",
                "time_signature": "4/4",
                "tags": ["rock", "demo"]
            ]
        ] as [String: Any]
        mockXPCClient.commitHistory = [commit]

        // When
        await viewModel.loadCommitHistory()

        // Then
        XCTAssertEqual(viewModel.commits.count, 1)
        let loadedCommit = viewModel.commits[0]
        XCTAssertNotNil(loadedCommit.metadata)
        XCTAssertEqual(loadedCommit.metadata?.bpm, 120.0)
        XCTAssertEqual(loadedCommit.metadata?.sampleRate, 44100)
        XCTAssertEqual(loadedCommit.metadata?.keySignature, "C Major")
        XCTAssertEqual(loadedCommit.metadata?.timeSignature, "4/4")
        XCTAssertEqual(loadedCommit.metadata?.tags, ["rock", "demo"])
    }

    func testLoadCommitHistory_WithBackwardCompatibleHash_ShouldParseCorrectly() async {
        // Given - Commit uses "hash" instead of "id" for backwards compatibility
        let commit = [
            "hash": "old123",
            "message": "Old commit format",
            "timestamp": Date(),
            "author": "Test User"
        ] as [String: Any]
        mockXPCClient.commitHistory = [commit]

        // When
        await viewModel.loadCommitHistory()

        // Then
        XCTAssertEqual(viewModel.commits.count, 1)
        XCTAssertEqual(viewModel.commits[0].id, "old123")
    }

    func testLoadCommitHistory_WhenEmpty_ShouldResultInEmptyCommitsList() async {
        // Given
        mockXPCClient.commitHistory = []

        // When
        await viewModel.loadCommitHistory()

        // Then
        XCTAssertTrue(viewModel.commits.isEmpty)
        XCTAssertFalse(viewModel.isLoading)
    }

    func testLoadCommitHistory_WithInvalidCommit_ShouldSkipInvalidEntries() async {
        // Given
        let validCommit = [
            "id": "abc123",
            "message": "Valid commit"
        ] as [String: Any]
        let invalidCommit = [
            "id": "def456"
            // Missing "message"
        ] as [String: Any]
        mockXPCClient.commitHistory = [validCommit, invalidCommit]

        // When
        await viewModel.loadCommitHistory()

        // Then
        XCTAssertEqual(viewModel.commits.count, 1)
        XCTAssertEqual(viewModel.commits[0].id, "abc123")
    }

    func testLoadCommitHistory_SetsLoadingState() async {
        // Given
        mockXPCClient.commitHistory = []

        // When/Then
        XCTAssertFalse(viewModel.isLoading)

        let loadTask = Task {
            await viewModel.loadCommitHistory()
        }

        // Should eventually finish loading
        await loadTask.value
        XCTAssertFalse(viewModel.isLoading)
    }

    // MARK: - Restore To Commit Tests

    func testRestoreToCommit_WhenSuccessful_ShouldReturnTrue() async {
        // Given
        let commit = CommitInfo(
            id: "abc123",
            message: "Test commit",
            timestamp: Date(),
            author: "Test User",
            metadata: nil
        )
        mockXPCClient.restoreResult = true

        // When
        let result = await viewModel.restoreToCommit(commit)

        // Then
        XCTAssertTrue(result)
        XCTAssertEqual(mockXPCClient.lastRestorePath, testProject.path)
        XCTAssertEqual(mockXPCClient.lastRestoreCommitHash, "abc123")
    }

    func testRestoreToCommit_WhenFails_ShouldReturnFalse() async {
        // Given
        let commit = CommitInfo(
            id: "abc123",
            message: "Test commit",
            timestamp: Date(),
            author: "Test User",
            metadata: nil
        )
        mockXPCClient.restoreResult = false

        // When
        let result = await viewModel.restoreToCommit(commit)

        // Then
        XCTAssertFalse(result)
    }

    // MARK: - Create Milestone Commit Tests

    func testCreateMilestoneCommit_WhenSuccessful_ShouldReturnTrue() async {
        // Given
        mockXPCClient.commitResult = true

        // When
        let result = await viewModel.createMilestoneCommit(
            message: "Milestone commit",
            metadata: nil
        )

        // Then
        XCTAssertTrue(result)
        XCTAssertEqual(mockXPCClient.lastCommitPath, testProject.path)
        XCTAssertEqual(mockXPCClient.lastCommitMessage, "Milestone commit")
        XCTAssertNotNil(mockXPCClient.lastCommitMetadata)
    }

    func testCreateMilestoneCommit_WithMetadata_ShouldPassMetadataCorrectly() async {
        // Given
        mockXPCClient.commitResult = true
        let metadata = CommitMetadata(
            bpm: 140.0,
            sampleRate: 48000,
            keySignature: "D Minor",
            timeSignature: "3/4",
            tags: ["EDM", "production"]
        )

        // When
        let result = await viewModel.createMilestoneCommit(
            message: "Production milestone",
            metadata: metadata
        )

        // Then
        XCTAssertTrue(result)
        let sentMetadata = mockXPCClient.lastCommitMetadata
        XCTAssertEqual(sentMetadata?["bpm"] as? Double, 140.0)
        XCTAssertEqual(sentMetadata?["sample_rate"] as? Int, 48000)
        XCTAssertEqual(sentMetadata?["key_signature"] as? String, "D Minor")
        XCTAssertEqual(sentMetadata?["time_signature"] as? String, "3/4")
        XCTAssertEqual(sentMetadata?["tags"] as? [String], ["EDM", "production"])
    }

    func testCreateMilestoneCommit_WhenSuccessful_ShouldReloadCommitHistory() async {
        // Given
        mockXPCClient.commitResult = true
        let newCommit = [
            "id": "new123",
            "message": "Production milestone",
            "timestamp": Date(),
            "author": "Test User"
        ] as [String: Any]
        mockXPCClient.commitHistory = [newCommit]

        // When
        let result = await viewModel.createMilestoneCommit(
            message: "Production milestone",
            metadata: nil
        )

        // Then
        XCTAssertTrue(result)
        XCTAssertEqual(viewModel.commits.count, 1)
        XCTAssertEqual(viewModel.commits[0].message, "Production milestone")
    }

    func testCreateMilestoneCommit_WhenFails_ShouldReturnFalseAndNotReload() async {
        // Given
        mockXPCClient.commitResult = false
        mockXPCClient.commitHistory = []

        // When
        let result = await viewModel.createMilestoneCommit(
            message: "Failed commit",
            metadata: nil
        )

        // Then
        XCTAssertFalse(result)
        XCTAssertTrue(viewModel.commits.isEmpty)
    }

    // MARK: - Project Property Tests

    func testInitialization_ShouldSetProjectCorrectly() {
        // Then
        XCTAssertEqual(viewModel.project.path, testProject.path)
        XCTAssertEqual(viewModel.project.projectType, testProject.projectType)
        XCTAssertTrue(viewModel.commits.isEmpty)
        XCTAssertFalse(viewModel.isLoading)
        XCTAssertNil(viewModel.errorMessage)
    }
}
