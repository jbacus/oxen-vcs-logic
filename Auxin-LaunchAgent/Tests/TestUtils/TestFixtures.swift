import Foundation

/// Test fixtures for creating Logic Pro project structures and test environments
public struct TestFixtures {
    /// Creates a temporary Logic Pro project with realistic structure
    ///
    /// - Parameters:
    ///   - name: Project name (default: "TestProject")
    ///   - bpm: Tempo in beats per minute (default: 120)
    ///   - sampleRate: Sample rate in Hz (default: 48000)
    ///   - audioFileSizeMB: Size of dummy audio file in MB (default: 1)
    ///
    /// - Returns: URL to the created project package
    public static func createLogicProject(
        name: String = "TestProject",
        bpm: Int = 120,
        sampleRate: Int = 48000,
        audioFileSizeMB: Int = 1
    ) -> URL {
        let tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString)

        do {
            try FileManager.default.createDirectory(
                at: tempDir,
                withIntermediateDirectories: true,
                attributes: nil
            )

            let projectPackage = tempDir.appendingPathComponent("\(name).logicx")
            try FileManager.default.createDirectory(
                at: projectPackage,
                withIntermediateDirectories: true,
                attributes: nil
            )

            // Create required Alternatives directory
            let alternatives = projectPackage.appendingPathComponent("Alternatives")
            try FileManager.default.createDirectory(
                at: alternatives,
                withIntermediateDirectories: true,
                attributes: nil
            )

            // Create Media directory
            let media = projectPackage.appendingPathComponent("Media")
            try FileManager.default.createDirectory(
                at: media,
                withIntermediateDirectories: true,
                attributes: nil
            )

            // Create dummy audio file
            let audioFile = media.appendingPathComponent("Audio_01.wav")
            let audioData = Data(count: audioFileSizeMB * 1024 * 1024)
            try audioData.write(to: audioFile)

            // Create projectData file with metadata
            let projectData = projectPackage.appendingPathComponent("projectData")
            let xml = """
            <?xml version="1.0" encoding="UTF-8"?>
            <project>
                <tempo>\(bpm)</tempo>
                <sampleRate>\(sampleRate)</sampleRate>
            </project>
            """
            try xml.write(to: projectData, atomically: true, encoding: .utf8)

            return projectPackage
        } catch {
            fatalError("Failed to create test Logic Pro project: \(error)")
        }
    }

    /// Creates a temporary directory for testing
    ///
    /// - Returns: URL to the created temporary directory
    public static func createTempDirectory() -> URL {
        let tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString)

        do {
            try FileManager.default.createDirectory(
                at: tempDir,
                withIntermediateDirectories: true,
                attributes: nil
            )
            return tempDir
        } catch {
            fatalError("Failed to create temp directory: \(error)")
        }
    }

    /// Creates a temporary file with specified content
    ///
    /// - Parameters:
    ///   - name: Filename
    ///   - content: File content
    ///
    /// - Returns: URL to the created file
    public static func createTempFile(name: String, content: String) -> URL {
        let tempDir = createTempDirectory()
        let fileURL = tempDir.appendingPathComponent(name)

        do {
            try content.write(to: fileURL, atomically: true, encoding: .utf8)
            return fileURL
        } catch {
            fatalError("Failed to create temp file: \(error)")
        }
    }

    /// Cleans up a temporary test project or directory
    ///
    /// - Parameter projectURL: URL to the project or directory to clean up
    public static func cleanup(_ projectURL: URL) {
        // Clean up the entire temp directory (parent of project package)
        let tempDir = projectURL.deletingLastPathComponent()

        do {
            if FileManager.default.fileExists(atPath: tempDir.path) {
                try FileManager.default.removeItem(at: tempDir)
            }
        } catch {
            // Log error but don't fail test
            NSLog("[TestFixtures] Cleanup warning: Failed to remove \(tempDir.path): \(error)")
        }
    }

    /// Creates multiple test projects for multi-project testing
    ///
    /// - Parameter count: Number of projects to create
    ///
    /// - Returns: Array of project URLs
    public static func createMultipleProjects(count: Int) -> [URL] {
        var projects: [URL] = []

        for i in 0..<count {
            let project = createLogicProject(
                name: "TestProject\(i + 1)",
                bpm: 120 + (i * 10),
                sampleRate: 48000
            )
            projects.append(project)
        }

        return projects
    }

    /// Cleans up multiple test projects
    ///
    /// - Parameter projects: Array of project URLs to clean up
    public static func cleanupMultiple(_ projects: [URL]) {
        for project in projects {
            cleanup(project)
        }
    }

    /// Creates a test lock file
    ///
    /// - Parameters:
    ///   - projectPath: Path to the project
    ///   - owner: Owner identifier
    ///   - timestamp: Lock timestamp (default: current time)
    ///
    /// - Returns: URL to the created lock file
    public static func createLockFile(
        projectPath: String,
        owner: String,
        timestamp: Date = Date()
    ) -> URL {
        let lockDir = createTempDirectory()
        let lockFile = lockDir.appendingPathComponent("locks.json")

        let lock: [String: Any] = [
            "project": projectPath,
            "owner": owner,
            "timestamp": ISO8601DateFormatter().string(from: timestamp),
            "pid": ProcessInfo.processInfo.processIdentifier
        ]

        do {
            let jsonData = try JSONSerialization.data(withJSONObject: lock, options: .prettyPrinted)
            try jsonData.write(to: lockFile)
            return lockFile
        } catch {
            fatalError("Failed to create lock file: \(error)")
        }
    }

    /// Modifies a file in a project to trigger FSEvents
    ///
    /// - Parameters:
    ///   - projectURL: URL to the project
    ///   - filename: Name of file to modify (created in Media directory)
    ///   - content: Content to write
    public static func modifyProjectFile(
        _ projectURL: URL,
        filename: String = "modified.txt",
        content: String = "Modified content"
    ) {
        let media = projectURL.appendingPathComponent("Media")
        let fileURL = media.appendingPathComponent(filename)

        do {
            try content.write(to: fileURL, atomically: true, encoding: .utf8)
        } catch {
            fatalError("Failed to modify project file: \(error)")
        }
    }

    /// Waits for a specified duration (useful for debounce testing)
    ///
    /// - Parameter seconds: Number of seconds to wait
    public static func wait(seconds: TimeInterval) {
        let expectation = XCTestExpectation(description: "Wait")
        DispatchQueue.main.asyncAfter(deadline: .now() + seconds) {
            expectation.fulfill()
        }
        _ = XCTWaiter.wait(for: [expectation], timeout: seconds + 1.0)
    }
}

// MARK: - XCTest Import
import XCTest
