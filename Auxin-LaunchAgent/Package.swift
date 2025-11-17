// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "Auxin-LaunchAgent",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .executable(
            name: "auxin-daemon",
            targets: ["Auxin-LaunchAgent"]
        ),
    ],
    dependencies: [],
    targets: [
        .executableTarget(
            name: "Auxin-LaunchAgent",
            dependencies: [],
            path: "Sources",
            resources: [
                .copy("../Resources")
            ]
        ),
        .testTarget(
            name: "Auxin-LaunchAgentTests",
            dependencies: ["Auxin-LaunchAgent"],
            path: "Tests"
        ),
    ]
)
