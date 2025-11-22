import XCTest
@testable import Auxin_App

final class ProjectTests: XCTestCase {

    func testProjectInitialization() {
        let path = "/Users/test/Music/My Awesome Song.logicx"
        let project = Project(path: path)

        XCTAssertEqual(project.path, path)
        XCTAssertEqual(project.id, path)
        XCTAssertEqual(project.name, "My Awesome Song.logicx")
        XCTAssertEqual(project.displayName, "My Awesome Song")
        XCTAssertEqual(project.projectType, .logicPro)
    }

    func testProjectTypeDetection() {
        let logicPath = "/path/to/Song.logicx"
        let sketchupPath = "/path/to/Model.skp"
        let blenderPath = "/path/to/Scene.blend"
        let unknownPath = "/path/to/document.txt"

        let logicProject = Project(path: logicPath)
        let sketchupProject = Project(path: sketchupPath)
        let blenderProject = Project(path: blenderPath)
        let unknownProject = Project(path: unknownPath)

        XCTAssertEqual(logicProject.projectType, .logicPro)
        XCTAssertEqual(sketchupProject.projectType, .sketchup)
        XCTAssertEqual(blenderProject.projectType, .blender)
        // It defaults to logicPro for unknown types, based on the model's init.
        XCTAssertEqual(unknownProject.projectType, .logicPro)
    }

    func testProjectEquality() {
        let project1 = Project(path: "/path/one")
        let project2 = Project(path: "/path/one")
        let project3 = Project(path: "/path/two")

        XCTAssertEqual(project1, project2)
        XCTAssertNotEqual(project1, project3)
    }

    func testProjectHashable() {
        let project1 = Project(path: "/path/one")
        let project2 = Project(path: "/path/one")

        var set = Set<Project>()
        set.insert(project1)
        set.insert(project2)

        XCTAssertEqual(set.count, 1)
    }
}
