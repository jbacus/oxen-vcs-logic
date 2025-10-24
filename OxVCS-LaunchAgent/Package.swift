// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "OxVCS-LaunchAgent",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .executable(
            name: "oxvcs-monitor",
            targets: ["OxVCS-LaunchAgent"]
        ),
    ],
    dependencies: [],
    targets: [
        .executableTarget(
            name: "OxVCS-LaunchAgent",
            dependencies: [],
            path: "Sources"
        ),
        .testTarget(
            name: "OxVCS-LaunchAgentTests",
            dependencies: ["OxVCS-LaunchAgent"],
            path: "Tests"
        ),
    ]
)
