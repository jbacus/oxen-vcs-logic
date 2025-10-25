// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "OxVCS-App",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .executable(
            name: "OxVCS",
            targets: ["OxVCS-App"]
        )
    ],
    dependencies: [],
    targets: [
        .executableTarget(
            name: "OxVCS-App",
            dependencies: [],
            path: "Sources"
        ),
        .testTarget(
            name: "OxVCS-AppTests",
            dependencies: ["OxVCS-App"],
            path: "Tests"
        )
    ]
)
