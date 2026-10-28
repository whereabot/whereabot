// swift-tools-version:5.3

import PackageDescription

let package = Package(
  name: "tauri-plugin-background-answer",
  platforms: [
    .iOS(.v14)
  ],
  products: [
    .library(
      name: "tauri-plugin-background-answer",
      type: .static,
      targets: ["tauri-plugin-background-answer"])
  ],
  dependencies: [
    .package(name: "Tauri", path: "../.tauri/tauri-api")
  ],
  targets: [
    .target(
      name: "tauri-plugin-background-answer",
      dependencies: [
        .byName(name: "Tauri")
      ],
      path: "Sources")
  ]
)
