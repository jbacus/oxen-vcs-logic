import Foundation
import CoreServices

@main
@available(macOS 13.0, *)
struct OxVCSDaemon {
    static func main() async {
        let args = CommandLine.arguments

        // Check macOS version
        if #available(macOS 13.0, *) {
            await OxenDaemon.main(arguments: args)
        } else {
            print("Error: Oxen VCS Daemon requires macOS 13.0 or later")
            print("Your version: \(ProcessInfo.processInfo.operatingSystemVersionString)")
            exit(1)
        }
    }
}
