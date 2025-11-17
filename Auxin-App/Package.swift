// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "Auxin-App",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .executable(
            name: "Auxin",
            targets: ["Auxin-App"]
        )
    ],
    dependencies: [],
    targets: [
        .executableTarget(
            name: "Auxin-App",
            dependencies: [],
            path: "Sources"
        ),
        .testTarget(
            name: "Auxin-AppTests",
            dependencies: ["Auxin-App"],
            path: "Tests"
        )
    ]
)
