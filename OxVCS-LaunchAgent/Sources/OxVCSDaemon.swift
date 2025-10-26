import Foundation
import CoreServices

@main
@available(macOS 13.0, *)
struct OxVCSDaemon {
    static func main() async {
        let args = CommandLine.arguments
        await OxenDaemon.main(arguments: args)
    }
}
