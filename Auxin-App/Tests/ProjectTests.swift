import XCTest
@testable import AuxinApp

/// Tests for the Project model
final class ProjectTests: XCTestCase {

    // MARK: - Initialization Tests

    func testProjectInitialization() {
        let project = Project(
            id: "test-id",
            name: "Test Project",
            path: "/Users/test/Music/Project.logicx",
            type: .logicPro
        )

        XCTAssertEqual(project.id, "test-id")
        XCTAssertEqual(project.name, "Test Project")
        XCTAssertEqual(project.path, "/Users/test/Music/Project.logicx")
        XCTAssertEqual(project.type, .logicPro)
    }

    func testProjectInitWithURL() {
        let url = URL(fileURLWithPath: "/Users/test/Models/Building.skp")
        let project = Project(url: url)

        XCTAssertEqual(project.path, url.path)
        XCTAssertEqual(project.type, .sketchUp)
    }

    func testProjectTypeDetection() {
        // Logic Pro
        let logicProject = Project(
            id: "1",
            name: "Song",
            path: "/path/to/Song.logicx",
            type: .auto
        )
        XCTAssertEqual(logicProject.detectedType, .logicPro)

        // SketchUp
        let sketchProject = Project(
            id: "2",
            name: "Model",
            path: "/path/to/Model.skp",
            type: .auto
        )
        XCTAssertEqual(sketchProject.detectedType, .sketchUp)

        // Blender
        let blenderProject = Project(
            id: "3",
            name: "Scene",
            path: "/path/to/Scene.blend",
            type: .auto
        )
        XCTAssertEqual(blenderProject.detectedType, .blender)
    }

    // MARK: - Status Tests

    func testProjectStatusInitial() {
        let project = Project(
            id: "test",
            name: "Test",
            path: "/path",
            type: .logicPro
        )

        XCTAssertEqual(project.status, .unknown)
        XCTAssertFalse(project.isLocked)
        XCTAssertFalse(project.hasUncommittedChanges)
    }

    func testProjectStatusLocked() {
        var project = Project(
            id: "test",
            name: "Test",
            path: "/path",
            type: .logicPro
        )

        project.lockStatus = .lockedByOther(user: "other@host")

        XCTAssertTrue(project.isLocked)
        XCTAssertFalse(project.isLockedByMe)
    }

    func testProjectStatusLockedByMe() {
        var project = Project(
            id: "test",
            name: "Test",
            path: "/path",
            type: .logicPro
        )

        project.lockStatus = .lockedByMe

        XCTAssertTrue(project.isLocked)
        XCTAssertTrue(project.isLockedByMe)
    }

    // MARK: - Metadata Tests

    func testProjectMetadataLogicPro() {
        var project = Project(
            id: "test",
            name: "Song",
            path: "/path/Song.logicx",
            type: .logicPro
        )

        project.metadata = ProjectMetadata(
            bpm: 128.0,
            sampleRate: 48000,
            key: "A Minor",
            tags: ["mixing", "vocals"]
        )

        XCTAssertEqual(project.metadata?.bpm, 128.0)
        XCTAssertEqual(project.metadata?.sampleRate, 48000)
        XCTAssertEqual(project.metadata?.key, "A Minor")
        XCTAssertEqual(project.metadata?.tags?.count, 2)
    }

    func testProjectMetadataSketchUp() {
        var project = Project(
            id: "test",
            name: "Model",
            path: "/path/Model.skp",
            type: .sketchUp
        )

        project.sketchUpMetadata = SketchUpMetadata(
            units: "Feet",
            layers: 15,
            components: 234,
            groups: 12
        )

        XCTAssertEqual(project.sketchUpMetadata?.units, "Feet")
        XCTAssertEqual(project.sketchUpMetadata?.layers, 15)
        XCTAssertEqual(project.sketchUpMetadata?.components, 234)
    }

    // MARK: - Commit History Tests

    func testProjectCommitHistory() {
        var project = Project(
            id: "test",
            name: "Test",
            path: "/path",
            type: .logicPro
        )

        let commits = [
            Commit(id: "abc123", message: "Initial", author: "user", date: Date()),
            Commit(id: "def456", message: "Added vocals", author: "user", date: Date())
        ]

        project.commits = commits

        XCTAssertEqual(project.commits.count, 2)
        XCTAssertEqual(project.latestCommit?.id, "def456")
    }

    // MARK: - Validation Tests

    func testProjectValidPath() {
        let project = Project(
            id: "test",
            name: "Valid",
            path: "/Users/test/Music/Valid.logicx",
            type: .logicPro
        )

        XCTAssertTrue(project.hasValidPath)
    }

    func testProjectInvalidPath() {
        let project = Project(
            id: "test",
            name: "Invalid",
            path: "",
            type: .logicPro
        )

        XCTAssertFalse(project.hasValidPath)
    }

    // MARK: - Equatable Tests

    func testProjectEquality() {
        let project1 = Project(id: "same", name: "A", path: "/a", type: .logicPro)
        let project2 = Project(id: "same", name: "B", path: "/b", type: .sketchUp)

        XCTAssertEqual(project1, project2) // Same ID
    }

    func testProjectInequality() {
        let project1 = Project(id: "1", name: "A", path: "/a", type: .logicPro)
        let project2 = Project(id: "2", name: "A", path: "/a", type: .logicPro)

        XCTAssertNotEqual(project1, project2) // Different IDs
    }

    // MARK: - Hashable Tests

    func testProjectHashable() {
        let project1 = Project(id: "hash", name: "A", path: "/a", type: .logicPro)
        let project2 = Project(id: "hash", name: "B", path: "/b", type: .sketchUp)

        var set = Set<Project>()
        set.insert(project1)
        set.insert(project2)

        XCTAssertEqual(set.count, 1) // Same hash, same ID
    }
}
