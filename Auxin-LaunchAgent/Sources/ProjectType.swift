import Foundation

/// Supported creative application project types
public enum ProjectType: String, Codable, CaseIterable {
    case logicPro = "logicpro"
    case sketchup = "sketchup"
    case blender = "blender"

    /// Display name for the project type
    public var displayName: String {
        switch self {
        case .logicPro:
            return "Logic Pro"
        case .sketchup:
            return "SketchUp"
        case .blender:
            return "Blender"
        }
    }

    /// File extension for the project type
    public var fileExtension: String {
        switch self {
        case .logicPro:
            return "logicx"
        case .sketchup:
            return "skp"
        case .blender:
            return "blend"
        }
    }

    /// Default search paths for projects of this type
    public var defaultSearchPaths: [String] {
        let homeDir = FileManager.default.homeDirectoryForCurrentUser.path

        switch self {
        case .logicPro:
            return [
                "\(homeDir)/Music",
                "\(homeDir)/Documents",
                "\(homeDir)/Desktop"
            ]
        case .sketchup:
            return [
                "\(homeDir)/Documents",
                "\(homeDir)/Desktop"
            ]
        case .blender:
            return [
                "\(homeDir)/Documents",
                "\(homeDir)/Desktop"
            ]
        }
    }

    /// Application bundle identifier for opening projects
    public var applicationBundleIdentifier: String? {
        switch self {
        case .logicPro:
            return "com.apple.logic10"
        case .sketchup:
            return "com.sketchup.SketchUp.2024"  // May vary by version
        case .blender:
            return "org.blenderfoundation.blender"
        }
    }

    /// Whether the project type uses folder-based structure (bundle)
    public var isFolderBased: Bool {
        switch self {
        case .logicPro:
            return true
        case .sketchup, .blender:
            return false
        }
    }

    /// Tracked directories within the project (for folder-based projects)
    public var trackedDirectories: [String] {
        switch self {
        case .logicPro:
            return ["Alternatives", "Resources"]
        case .sketchup:
            return ["textures", "components", "materials"]
        case .blender:
            return ["textures", "libraries", "assets", "scripts"]
        }
    }

    /// Ignored directories/patterns for FSEvents filtering
    public var ignoredPatterns: [String] {
        switch self {
        case .logicPro:
            return ["Bounces", "Freeze Files", "Autosave", ".DS_Store"]
        case .sketchup:
            return [".skb", "exports", "renders", ".thumbnails", ".DS_Store"]
        case .blender:
            return [".blend1", "blendcache_", "renders", "__pycache__", ".DS_Store"]
        }
    }

    /// Key file patterns to monitor (triggers auto-commit)
    public var keyFilePatterns: [String] {
        switch self {
        case .logicPro:
            return ["projectData", "ProjectData"]
        case .sketchup:
            return [".skp"]
        case .blender:
            return [".blend"]
        }
    }

    /// Folders to clean up before milestone commits
    public var cleanupFolders: [String] {
        switch self {
        case .logicPro:
            return ["Bounces", "Freeze Files", "Media.localized"]
        case .sketchup:
            return ["exports", "renders"]
        case .blender:
            return ["renders", "blendcache_"]
        }
    }

    /// CLI argument for auxin init command
    public var cliArgument: String {
        switch self {
        case .logicPro:
            return "--type logicpro"
        case .sketchup:
            return "--type sketchup"
        case .blender:
            return "--type blender"
        }
    }

    /// Detect project type from file path
    public static func detect(from path: String) -> ProjectType? {
        let url = URL(fileURLWithPath: path)
        let ext = url.pathExtension.lowercased()

        switch ext {
        case "logicx":
            return .logicPro
        case "skp":
            return .sketchup
        case "blend":
            return .blender
        default:
            return nil
        }
    }

    /// All supported file extensions
    public static var supportedExtensions: [String] {
        return allCases.map { $0.fileExtension }
    }

    /// Check if a path is a supported project type
    public static func isSupported(_ path: String) -> Bool {
        return detect(from: path) != nil
    }
}

/// Project info with type information
public struct ProjectInfo {
    public let path: String
    public let type: ProjectType
    public let name: String

    public init(path: String, type: ProjectType) {
        self.path = path
        self.type = type
        self.name = URL(fileURLWithPath: path).lastPathComponent
    }

    public var displayName: String {
        name.replacingOccurrences(of: ".\(type.fileExtension)", with: "")
    }
}
