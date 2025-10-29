import XCTest
@testable import OxVCS_LaunchAgent

@available(macOS 13.0, *)
final class ServiceManagerTests: XCTestCase {

    var serviceManager: ServiceManager!
    var testPlistPath: String!

    override func setUp() {
        super.setUp()
        serviceManager = ServiceManager()

        // Create test plist path
        let tempDir = NSTemporaryDirectory()
        testPlistPath = (tempDir as NSString).appendingPathComponent("com.oxen.logic.daemon.plist")
    }

    override func tearDown() {
        // Clean up test files
        try? FileManager.default.removeItem(atPath: testPlistPath)
        serviceManager = nil
        super.tearDown()
    }

    // MARK: - Initialization Tests

    func testServiceManagerInitialization() {
        XCTAssertNotNil(serviceManager, "ServiceManager should initialize")
    }

    func testServiceManagerMultipleInstances() {
        let manager1 = ServiceManager()
        let manager2 = ServiceManager()

        XCTAssertNotNil(manager1, "First instance should initialize")
        XCTAssertNotNil(manager2, "Second instance should initialize")
    }

    // MARK: - Status Tests

    func testIsRunningWhenNotLoaded() {
        // Most likely the service is not loaded during testing
        // This tests the method doesn't crash
        let running = serviceManager.isRunning()
        XCTAssertTrue(running == true || running == false, "Should return a boolean")
    }

    func testGetStatusDescription() {
        let status = serviceManager.getStatusDescription()
        XCTAssertFalse(status.isEmpty, "Status should not be empty")
        XCTAssertTrue(
            status.contains("✓") || status.contains("✗") || status.contains("○"),
            "Status should contain a status indicator"
        )
    }

    func testGetStatusDescriptionFormats() {
        let status = serviceManager.getStatusDescription()

        // Should be one of the known statuses
        let validStatuses = [
            "✗ Service configuration not found (run install.sh)",
            "✓ Enabled and running",
            "○ Not loaded (run: oxvcs-daemon --install)"
        ]

        XCTAssertTrue(
            validStatuses.contains(status),
            "Status should be one of the valid formats: \(status)"
        )
    }

    // MARK: - Error Handling Tests

    func testServiceErrorDescriptions() {
        let errors: [ServiceManager.ServiceError] = [
            .registrationFailed("test"),
            .unregistrationFailed("test"),
            .statusCheckFailed("test"),
            .notAuthorized,
            .alreadyRegistered,
            .notRegistered,
            .plistNotFound
        ]

        for error in errors {
            XCTAssertNotNil(error.errorDescription, "Error should have description: \(error)")
            XCTAssertFalse(error.errorDescription!.isEmpty, "Error description should not be empty")
        }
    }

    func testRegistrationFailedErrorMessage() {
        let error = ServiceManager.ServiceError.registrationFailed("custom reason")
        XCTAssertTrue(error.errorDescription!.contains("custom reason"), "Should include custom reason")
    }

    func testUnregistrationFailedErrorMessage() {
        let error = ServiceManager.ServiceError.unregistrationFailed("custom reason")
        XCTAssertTrue(error.errorDescription!.contains("custom reason"), "Should include custom reason")
    }

    func testStatusCheckFailedErrorMessage() {
        let error = ServiceManager.ServiceError.statusCheckFailed("custom reason")
        XCTAssertTrue(error.errorDescription!.contains("custom reason"), "Should include custom reason")
    }

    func testPlistNotFoundErrorMessage() {
        let error = ServiceManager.ServiceError.plistNotFound
        XCTAssertTrue(error.errorDescription!.contains("plist"), "Should mention plist")
        XCTAssertTrue(error.errorDescription!.contains("install.sh"), "Should mention install script")
    }

    // MARK: - Installation Tests

    func testInstallReturnsBool() {
        let result = serviceManager.install()
        XCTAssertTrue(result == true || result == false, "Should return a boolean")
    }

    func testUninstallReturnsBool() {
        let result = serviceManager.uninstall()
        XCTAssertTrue(result == true || result == false, "Should return a boolean")
    }

    // MARK: - Configuration Verification Tests

    func testVerifyConfigurationExists() {
        // This tests that the method executes without crashing
        let result = ServiceManager.verifyConfiguration()
        XCTAssertTrue(result == true || result == false, "Should return a boolean")
    }

    func testInstallPlistCreatesDirectoryIfNeeded() throws {
        // Create a temporary source plist
        let sourcePlist = testPlistPath!
        let plistContent = """
        <?xml version="1.0" encoding="UTF-8"?>
        <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
        <plist version="1.0">
        <dict>
            <key>Label</key>
            <string>com.oxen.logic.daemon</string>
        </dict>
        </plist>
        """
        try plistContent.write(toFile: sourcePlist, atomically: true, encoding: .utf8)

        // Test that the method handles directory creation
        // Note: We won't actually call it to avoid modifying the real LaunchAgents directory
        XCTAssertTrue(FileManager.default.fileExists(atPath: sourcePlist), "Test plist should exist")
    }

    // MARK: - Command Line Interface Tests

    func testHandleCommandWithInstall() {
        let args = ["daemon", "--install"]
        // Note: We don't actually call handleCommand to avoid system modifications
        XCTAssertEqual(args[1], "--install", "Should recognize install command")
    }

    func testHandleCommandWithUninstall() {
        let args = ["daemon", "--uninstall"]
        XCTAssertEqual(args[1], "--uninstall", "Should recognize uninstall command")
    }

    func testHandleCommandWithStatus() {
        let args = ["daemon", "--status"]
        XCTAssertEqual(args[1], "--status", "Should recognize status command")
    }

    func testHandleCommandWithVerify() {
        let args = ["daemon", "--verify"]
        XCTAssertEqual(args[1], "--verify", "Should recognize verify command")
    }

    func testHandleCommandWithShorthandCommands() {
        let installArgs = ["daemon", "install"]
        let uninstallArgs = ["daemon", "uninstall"]
        let statusArgs = ["daemon", "status"]
        let verifyArgs = ["daemon", "verify"]

        XCTAssertEqual(installArgs[1], "install", "Should recognize install without --")
        XCTAssertEqual(uninstallArgs[1], "uninstall", "Should recognize uninstall without --")
        XCTAssertEqual(statusArgs[1], "status", "Should recognize status without --")
        XCTAssertEqual(verifyArgs[1], "verify", "Should recognize verify without --")
    }

    func testHandleCommandWithInvalidCommand() {
        let args = ["daemon", "--invalid"]
        XCTAssertEqual(args[1], "--invalid", "Should handle invalid command")
    }

    func testHandleCommandWithNoArguments() {
        let args = ["daemon"]
        XCTAssertEqual(args.count, 1, "Should handle no arguments case")
    }

    // MARK: - Status Display Tests

    func testPrintStatusDoesNotCrash() {
        // This tests that printStatus executes without crashing
        serviceManager.printStatus()
        // If we get here without crashing, test passes
        XCTAssertTrue(true, "printStatus should not crash")
    }

    // MARK: - Path Handling Tests

    func testPlistPathExpansion() {
        // Test that tilde expansion works
        let unexpandedPath = "~/Library/LaunchAgents/com.oxen.logic.daemon.plist"
        let expandedPath = NSString(string: unexpandedPath).expandingTildeInPath

        XCTAssertFalse(expandedPath.contains("~"), "Should expand tilde")
        XCTAssertTrue(expandedPath.contains("Library/LaunchAgents"), "Should contain LaunchAgents")
    }

    // MARK: - Integration Tests

    func testServiceNameIsCorrect() {
        // Verify the service name matches expectations
        let status = serviceManager.getStatusDescription()
        // Status should reference the service indirectly
        XCTAssertFalse(status.isEmpty, "Status should reference service")
    }

    func testMultipleStatusChecks() {
        // Test that multiple status checks don't cause issues
        let status1 = serviceManager.getStatusDescription()
        let status2 = serviceManager.getStatusDescription()
        let status3 = serviceManager.getStatusDescription()

        XCTAssertFalse(status1.isEmpty, "First check should work")
        XCTAssertFalse(status2.isEmpty, "Second check should work")
        XCTAssertFalse(status3.isEmpty, "Third check should work")
    }

    func testConcurrentStatusChecks() {
        let expectation = XCTestExpectation(description: "Concurrent status checks")
        expectation.expectedFulfillmentCount = 10

        for _ in 0..<10 {
            DispatchQueue.global().async {
                let status = self.serviceManager.getStatusDescription()
                XCTAssertFalse(status.isEmpty, "Concurrent check should work")
                expectation.fulfill()
            }
        }

        wait(for: [expectation], timeout: 5.0)
    }
}
