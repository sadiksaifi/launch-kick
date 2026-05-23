// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "LaunchKick",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .executable(name: "launch-kick", targets: ["LaunchKick"])
    ],
    targets: [
        .executableTarget(name: "LaunchKick"),
        .testTarget(
            name: "LaunchKickTests",
            dependencies: ["LaunchKick"]
        )
    ]
)
