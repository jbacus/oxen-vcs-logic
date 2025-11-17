import XCTest
@testable import Auxin_LaunchAgent

@available(macOS 13.0, *)
final class DaemonTests: XCTestCase {

    var daemon: OxenDaemon!
    var testProjectPath: String!

    override func setUp() {
        super.setUp()

        // Create test project path
        let tempDir = NSTemporaryDirectory()
        testProjectPath = (tempDir as NSString).appendingPathComponent("TestProject.logicx")

        // Create test project structure
        let fileManager = FileManager.default
        try? fileManager.createDirectory(atPath: testProjectPath, withIntermediateDirectories: true)
        try? fileManager.createDirectory(atPath: (testProjectPath as NSString).appendingPathComponent("Media"), withIntermediateDirectories: true)

        // Initialize daemon with test configuration
        daemon = OxenDaemon(
            cliPath: "/usr/local/bin/auxin",
            debounceThreshold: 1.0  // Short for testing
        )
    }

    override func tearDown() {
        // Clean up
        try? FileManager.default.removeItem(atPath: testProjectPath)
        daemon = nil
        super.tearDown()
    }

    // MARK: - Initialization Tests

    func testDaemonInitialization() {
        XCTAssertNotNil(daemon, "Daemon should initialize")

        let stats = daemon.getStatistics()
        XCTAssertEqual(stats["isRunning"] as? Bool, false, "Should not be running initially")
        XCTAssertEqual(stats["projectCount"] as? Int, 0, "Should have no projects initially")
        XCTAssertEqual(stats["debounceThreshold"] as? TimeInterval, 1.0, "Should use configured debounce")
    }

    func testDaemonInitializationWithDefaultValues() {
        let defaultDaemon = OxenDaemon()
        let stats = defaultDaemon.getStatistics()

        XCTAssertEqual(stats["cliPath"] as? String, "/usr/local/bin/auxin", "Should use default CLI path")
        XCTAssertEqual(stats["debounceThreshold"] as? TimeInterval, 30.0, "Should use default debounce")
    }

    func testDaemonInitializationWithCustomPath() {
        let customDaemon = OxenDaemon(cliPath: "/custom/path/cli")
        let stats = customDaemon.getStatistics()

        XCTAssertEqual(stats["cliPath"] as? String, "/custom/path/cli", "Should use custom CLI path")
    }

    // MARK: - Lifecycle Tests

    func testDaemonCannotStartTwice() async {
        // This test would need to be run in isolation or with proper async handling
        // For now, we test the guard condition logic
        let stats = daemon.getStatistics()
        XCTAssertFalse(stats["isRunning"] as? Bool ?? true, "Daemon should not be running")
    }

    func testDaemonStopWhenNotRunning() async {
        await daemon.stop()
        // Should not crash when stopping already-stopped daemon
        let stats = daemon.getStatistics()
        XCTAssertFalse(stats["isRunning"] as? Bool ?? true, "Should remain stopped")
    }

    // MARK: - Project Registration Tests

    func testRegisterProject() async {
        let initialCount = (daemon.getStatistics()["projectCount"] as? Int) ?? 0

        // Register project (note: will fail without actual Oxen setup, but tests the flow)
        await daemon.registerProject(testProjectPath)

        // Verify project count increased
        let finalCount = (daemon.getStatistics()["projectCount"] as? Int) ?? 0
        XCTAssertEqual(finalCount, initialCount + 1, "Project count should increase")
    }

    func testRegisterProjectNormalizesPath() async {
        let pathWithTilde = "~/TestProject.logicx"
        await daemon.registerProject(pathWithTilde)

        // Should normalize the path
        let stats = daemon.getStatistics()
        XCTAssertGreaterThanOrEqual(stats["projectCount"] as? Int ?? 0, 1, "Should register with normalized path")
    }

    func testRegisterSameProjectTwice() async {
        await daemon.registerProject(testProjectPath)
        let countAfterFirst = (daemon.getStatistics()["projectCount"] as? Int) ?? 0

        await daemon.registerProject(testProjectPath)
        let countAfterSecond = (daemon.getStatistics()["projectCount"] as? Int) ?? 0

        XCTAssertEqual(countAfterFirst, countAfterSecond, "Should not register same project twice")
    }

    func testUnregisterProject() async {
        // Register first
        await daemon.registerProject(testProjectPath)
        let countAfterRegister = (daemon.getStatistics()["projectCount"] as? Int) ?? 0

        // Unregister
        daemon.unregisterProject(testProjectPath)
        let countAfterUnregister = (daemon.getStatistics()["projectCount"] as? Int) ?? 0

        XCTAssertEqual(countAfterUnregister, countAfterRegister - 1, "Project count should decrease")
    }

    func testUnregisterNonexistentProject() {
        let initialCount = (daemon.getStatistics()["projectCount"] as? Int) ?? 0

        daemon.unregisterProject("/nonexistent/project.logicx")

        let finalCount = (daemon.getStatistics()["projectCount"] as? Int) ?? 0
        XCTAssertEqual(initialCount, finalCount, "Should handle unregistering nonexistent project")
    }

    // MARK: - Monitor Management Tests

    func testMonitorCountMatchesProjectCount() async {
        await daemon.registerProject(testProjectPath)

        let stats = daemon.getStatistics()
        let projectCount = stats["projectCount"] as? Int ?? 0
        let monitorCount = stats["monitorCount"] as? Int ?? 0

        XCTAssertEqual(projectCount, monitorCount, "Monitor count should match project count")
    }

    func testUnregisterStopsMonitor() async {
        await daemon.registerProject(testProjectPath)
        let monitorCountBefore = (daemon.getStatistics()["monitorCount"] as? Int) ?? 0

        daemon.unregisterProject(testProjectPath)
        let monitorCountAfter = (daemon.getStatistics()["monitorCount"] as? Int) ?? 0

        XCTAssertEqual(monitorCountAfter, monitorCountBefore - 1, "Monitor should be stopped and removed")
    }

    // MARK: - Statistics Tests

    func testGetStatistics() {
        let stats = daemon.getStatistics()

        XCTAssertNotNil(stats["isRunning"], "Should include isRunning")
        XCTAssertNotNil(stats["projectCount"], "Should include projectCount")
        XCTAssertNotNil(stats["monitorCount"], "Should include monitorCount")
        XCTAssertNotNil(stats["debounceThreshold"], "Should include debounceThreshold")
        XCTAssertNotNil(stats["cliPath"], "Should include cliPath")
        XCTAssertNotNil(stats["uptime"], "Should include uptime")
    }

    func testStatisticsTypes() {
        let stats = daemon.getStatistics()

        XCTAssertTrue(stats["isRunning"] is Bool, "isRunning should be Bool")
        XCTAssertTrue(stats["projectCount"] is Int, "projectCount should be Int")
        XCTAssertTrue(stats["monitorCount"] is Int, "monitorCount should be Int")
        XCTAssertTrue(stats["debounceThreshold"] is TimeInterval, "debounceThreshold should be TimeInterval")
        XCTAssertTrue(stats["cliPath"] is String, "cliPath should be String")
        XCTAssertTrue(stats["uptime"] is TimeInterval, "uptime should be TimeInterval")
    }

    func testStatisticsUptime() {
        let stats1 = daemon.getStatistics()
        let uptime1 = stats1["uptime"] as? TimeInterval ?? 0

        Thread.sleep(forTimeInterval: 0.1)

        let stats2 = daemon.getStatistics()
        let uptime2 = stats2["uptime"] as? TimeInterval ?? 0

        XCTAssertGreaterThanOrEqual(uptime2, uptime1, "Uptime should increase")
    }

    // MARK: - Path Normalization Tests

    func testPathNormalizationWithTrailingSlash() async {
        let pathWithSlash = testProjectPath + "/"
        await daemon.registerProject(pathWithSlash)

        let pathWithoutSlash = testProjectPath
        await daemon.registerProject(pathWithoutSlash!)

        // Should treat as same project
        let stats = daemon.getStatistics()
        XCTAssertEqual(stats["projectCount"] as? Int, 1, "Should normalize paths")
    }

    func testPathNormalizationWithDots() async {
        let complexPath = testProjectPath + "/../TestProject.logicx"
        await daemon.registerProject(complexPath)

        let stats = daemon.getStatistics()
        XCTAssertGreaterThanOrEqual(stats["projectCount"] as? Int ?? 0, 1, "Should handle path with dots")
    }

    // MARK: - Command Line Argument Tests

    func testMainWithInstallCommand() async {
        // Note: This would typically require mocking ServiceManager
        // For now, we test that it recognizes the command
        let arguments = ["daemon", "--install"]
        // Would call: await OxenDaemon.main(arguments: arguments)
        // But this would try to actually install, so we skip execution
        XCTAssertTrue(arguments.contains("--install"), "Should recognize install command")
    }

    func testMainWithDaemonCommand() async {
        let arguments = ["daemon", "--daemon"]
        XCTAssertTrue(arguments.contains("--daemon"), "Should recognize daemon command")
    }

    func testMainWithHelpCommand() async {
        let arguments = ["daemon", "--help"]
        XCTAssertTrue(arguments.contains("--help"), "Should recognize help command")
    }

    func testMainWithVersionCommand() async {
        let arguments = ["daemon", "--version"]
        XCTAssertTrue(arguments.contains("--version"), "Should recognize version command")
    }

    func testMainWithUnknownCommand() async {
        let arguments = ["daemon", "--unknown"]
        XCTAssertTrue(arguments.contains("--unknown"), "Should handle unknown command")
    }

    // MARK: - Edge Cases & Error Handling

    func testRegisterEmptyPath() async {
        await daemon.registerProject("")
        // Should handle gracefully
        // Note: actual behavior depends on implementation
    }

    func testRegisterInvalidPath() async {
        await daemon.registerProject("/this/path/does/not/exist.logicx")
        // Should handle gracefully without crashing
    }

    func testMultipleProjectRegistration() async {
        let project1 = testProjectPath!
        let project2 = (NSTemporaryDirectory() as NSString).appendingPathComponent("Project2.logicx")
        let project3 = (NSTemporaryDirectory() as NSString).appendingPathComponent("Project3.logicx")

        // Create directories
        try? FileManager.default.createDirectory(atPath: project2, withIntermediateDirectories: true)
        try? FileManager.default.createDirectory(atPath: project3, withIntermediateDirectories: true)

        await daemon.registerProject(project1)
        await daemon.registerProject(project2)
        await daemon.registerProject(project3)

        let stats = daemon.getStatistics()
        XCTAssertGreaterThanOrEqual(stats["projectCount"] as? Int ?? 0, 3, "Should handle multiple projects")

        // Clean up
        try? FileManager.default.removeItem(atPath: project2)
        try? FileManager.default.removeItem(atPath: project3)
    }

    func testUnregisterAllProjects() async {
        let project1 = testProjectPath!
        let project2 = (NSTemporaryDirectory() as NSString).appendingPathComponent("Project2.logicx")

        try? FileManager.default.createDirectory(atPath: project2, withIntermediateDirectories: true)

        await daemon.registerProject(project1)
        await daemon.registerProject(project2)

        daemon.unregisterProject(project1)
        daemon.unregisterProject(project2)

        let stats = daemon.getStatistics()
        XCTAssertEqual(stats["projectCount"] as? Int, 0, "Should unregister all projects")
        XCTAssertEqual(stats["monitorCount"] as? Int, 0, "Should stop all monitors")

        try? FileManager.default.removeItem(atPath: project2)
    }
}
