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

    /// SF Symbol icon name for the project type
    public var iconName: String {
        switch self {
        case .logicPro:
            return "music.note"
        case .sketchup:
            return "cube"
        case .blender:
            return "cube.transparent"
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

struct Project: Identifiable, Codable, Hashable {
    let id: String  // Stable identifier - use path
    let path: String
    let name: String
    let projectType: ProjectType
    let isMonitored: Bool
    let lastCommit: Date?
    let commitCount: Int
    let isLocked: Bool
    let lockedBy: String?

    var displayName: String {
        name.replacingOccurrences(of: ".\(projectType.fileExtension)", with: "")
    }

    var directoryURL: URL {
        URL(fileURLWithPath: path)
    }

    /// Icon name for this project type
    var iconName: String {
        projectType.iconName
    }

    /// Initialize with auto-detected project type
    init(path: String,
         name: String? = nil,
         projectType: ProjectType? = nil,
         isMonitored: Bool = false,
         lastCommit: Date? = nil,
         commitCount: Int = 0,
         isLocked: Bool = false,
         lockedBy: String? = nil) {
        self.id = path  // Use path as stable identifier
        self.path = path
        self.name = name ?? URL(fileURLWithPath: path).lastPathComponent
        self.projectType = projectType ?? ProjectType.detect(from: path) ?? .logicPro
        self.isMonitored = isMonitored
        self.lastCommit = lastCommit
        self.commitCount = commitCount
        self.isLocked = isLocked
        self.lockedBy = lockedBy
    }
}

struct CommitInfo: Identifiable, Codable {
    let id: String  // commit hash
    let message: String
    let timestamp: Date
    let author: String
    let metadata: CommitMetadata?

    var shortHash: String {
        String(id.prefix(7))
    }

    var formattedDate: String {
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        formatter.timeStyle = .short
        return formatter.string(from: timestamp)
    }
}

/// Generic commit metadata that can hold project-type-specific fields
struct CommitMetadata: Codable {
    // Logic Pro fields
    let bpm: Double?
    let sampleRate: Int?
    let keySignature: String?
    let timeSignature: String?

    // SketchUp fields
    let units: String?
    let layerCount: Int?
    let componentCount: Int?
    let groupCount: Int?

    // Blender fields
    let sceneCount: Int?
    let objectCount: Int?
    let materialCount: Int?
    let renderEngine: String?
    let resolution: String?
    let fps: Int?
    let frameRange: String?

    // Common fields
    let tags: [String]?
    let fileSizeBytes: Int?

    // Media attachments
    let thumbnailPath: String?
    let bouncePath: String?
    let screenshotPath: String?

    /// Default initializer with all optional parameters
    init(bpm: Double? = nil, sampleRate: Int? = nil, keySignature: String? = nil, timeSignature: String? = nil,
         units: String? = nil, layerCount: Int? = nil, componentCount: Int? = nil, groupCount: Int? = nil,
         sceneCount: Int? = nil, objectCount: Int? = nil, materialCount: Int? = nil,
         renderEngine: String? = nil, resolution: String? = nil, fps: Int? = nil, frameRange: String? = nil,
         tags: [String]? = nil, fileSizeBytes: Int? = nil,
         thumbnailPath: String? = nil, bouncePath: String? = nil, screenshotPath: String? = nil) {
        self.bpm = bpm
        self.sampleRate = sampleRate
        self.keySignature = keySignature
        self.timeSignature = timeSignature
        self.units = units
        self.layerCount = layerCount
        self.componentCount = componentCount
        self.groupCount = groupCount
        self.sceneCount = sceneCount
        self.objectCount = objectCount
        self.materialCount = materialCount
        self.renderEngine = renderEngine
        self.resolution = resolution
        self.fps = fps
        self.frameRange = frameRange
        self.tags = tags
        self.fileSizeBytes = fileSizeBytes
        self.thumbnailPath = thumbnailPath
        self.bouncePath = bouncePath
        self.screenshotPath = screenshotPath
    }

    /// Initialize with Logic Pro metadata
    static func logicPro(bpm: Double?, sampleRate: Int?, keySignature: String?, timeSignature: String?, tags: [String]?) -> CommitMetadata {
        CommitMetadata(
            bpm: bpm, sampleRate: sampleRate, keySignature: keySignature, timeSignature: timeSignature,
            units: nil, layerCount: nil, componentCount: nil, groupCount: nil,
            sceneCount: nil, objectCount: nil, materialCount: nil, renderEngine: nil, resolution: nil, fps: nil, frameRange: nil,
            tags: tags, fileSizeBytes: nil
        )
    }

    /// Initialize with SketchUp metadata
    static func sketchup(units: String?, layerCount: Int?, componentCount: Int?, groupCount: Int?, tags: [String]?) -> CommitMetadata {
        CommitMetadata(
            bpm: nil, sampleRate: nil, keySignature: nil, timeSignature: nil,
            units: units, layerCount: layerCount, componentCount: componentCount, groupCount: groupCount,
            sceneCount: nil, objectCount: nil, materialCount: nil, renderEngine: nil, resolution: nil, fps: nil, frameRange: nil,
            tags: tags, fileSizeBytes: nil
        )
    }

    /// Initialize with Blender metadata
    static func blender(sceneCount: Int?, objectCount: Int?, materialCount: Int?, renderEngine: String?, resolution: String?, fps: Int?, frameRange: String?, tags: [String]?) -> CommitMetadata {
        CommitMetadata(
            bpm: nil, sampleRate: nil, keySignature: nil, timeSignature: nil,
            units: nil, layerCount: nil, componentCount: nil, groupCount: nil,
            sceneCount: sceneCount, objectCount: objectCount, materialCount: materialCount, renderEngine: renderEngine, resolution: resolution, fps: fps, frameRange: frameRange,
            tags: tags, fileSizeBytes: nil
        )
    }
}

struct DaemonStatus: Codable {
    let isRunning: Bool
    let monitoredProjectCount: Int
    let lastActivity: Date?
}

struct ProjectLock: Codable {
    let projectPath: String
    let lockedBy: String
    let lockId: String
    let acquiredAt: Date
    let expiresAt: Date

    var isExpired: Bool {
        Date() > expiresAt
    }

    var remainingTime: TimeInterval {
        expiresAt.timeIntervalSince(Date())
    }
}
