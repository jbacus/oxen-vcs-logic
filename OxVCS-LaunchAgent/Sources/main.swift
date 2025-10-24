import Foundation
import CoreServices

@main
struct OxVCSMonitor {
    static func main() async {
        let monitor = FSEventsMonitor()

        // Parse command line arguments
        let args = CommandLine.arguments

        guard args.count > 1 else {
            print("Usage: oxvcs-monitor <path-to-logic-project>")
            print("Example: oxvcs-monitor ~/Music/MyProject.logicx")
            exit(1)
        }

        let projectPath = args[1]

        // Validate path exists and is a Logic Pro project
        guard FileManager.default.fileExists(atPath: projectPath) else {
            print("Error: Path does not exist: \(projectPath)")
            exit(1)
        }

        guard projectPath.hasSuffix(".logicx") else {
            print("Error: Path must be a Logic Pro folder project (.logicx)")
            exit(1)
        }

        print("🎵 OxVCS Monitor started")
        print("Watching: \(projectPath)")
        print("Debounce threshold: 30 seconds")
        print("Press Ctrl+C to stop\n")

        do {
            try await monitor.start(watchingPath: projectPath)
        } catch {
            print("Error: \(error.localizedDescription)")
            exit(1)
        }
    }
}
