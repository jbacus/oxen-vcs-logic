import Foundation
import SwiftUI

enum DaemonState: Equatable {
    case running
    case restarting
    case stopped
    case updating
    case checking

    var displayText: String {
        switch self {
        case .running: return "Running"
        case .restarting: return "Restarting"
        case .stopped: return "Stopped"
        case .updating: return "Updating"
        case .checking: return "Checking..."
        }
    }

    var color: Color {
        switch self {
        case .running: return .green
        case .restarting: return .orange
        case .stopped: return .red
        case .updating: return .blue
        case .checking: return .gray
        }
    }

    var needsProgressIndicator: Bool {
        switch self {
        case .restarting, .updating, .checking:
            return true
        case .running, .stopped:
            return false
        }
    }
}

// MARK: - Notification Names

extension Notification.Name {
    static let daemonStateChanged = Notification.Name("daemonStateChanged")
}
