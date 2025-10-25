// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "OxVCS-LaunchAgent",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .executable(
            name: "oxvcs-daemon",
            targets: ["OxVCS-LaunchAgent"]
        ),
    ],
    dependencies: [],
    targets: [
        .executableTarget(
            name: "OxVCS-LaunchAgent",
            dependencies: [],
            path: "Sources",
            resources: [
                .copy("../Resources")
            ]
        ),
        .testTarget(
            name: "OxVCS-LaunchAgentTests",
            dependencies: ["OxVCS-LaunchAgent"],
            path: "Tests"
        ),
    ]
)
