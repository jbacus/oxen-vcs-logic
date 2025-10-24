import Foundation
import CoreServices

/// FSEvents-based file system monitor with debounce logic
class FSEventsMonitor {

    // MARK: - Configuration

    /// Debounce threshold in seconds (default: 30 seconds)
    private let debounceThreshold: TimeInterval = 30.0

    /// Minimum delay between checks (default: 5 seconds)
    private let minimumCheckInterval: TimeInterval = 5.0

    // MARK: - State

    private var eventStream: FSEventStreamRef?
    private var lastEventTime: Date?
    private var debounceTimer: Timer?
    private var isMonitoring = false
    private var watchedPath: String = ""

    // MARK: - Lifecycle

    init() {}

    deinit {
        stop()
    }

    // MARK: - Public Methods

    /// Starts monitoring the specified path for file system events
    func start(watchingPath path: String) async throws {
        guard !isMonitoring else {
            print("⚠️  Already monitoring")
            return
        }

        self.watchedPath = path

        // Create FSEvents stream
        var context = FSEventStreamContext(
            version: 0,
            info: Unmanaged.passUnretained(self).toOpaque(),
            retain: nil,
            release: nil,
            copyDescription: nil
        )

        let pathsToWatch = [path] as CFArray
        let flags = UInt32(kFSEventStreamCreateFlagFileEvents | kFSEventStreamCreateFlagUseCFTypes)

        guard let stream = FSEventStreamCreate(
            kCFAllocatorDefault,
            { (
                streamRef,
                clientCallBackInfo,
                numEvents,
                eventPaths,
                eventFlags,
                eventIds
            ) in
                guard let info = clientCallBackInfo else { return }
                let monitor = Unmanaged<FSEventsMonitor>.fromOpaque(info).takeUnretainedValue()
                monitor.handleEvents(
                    numEvents: numEvents,
                    eventPaths: eventPaths,
                    eventFlags: eventFlags,
                    eventIds: eventIds
                )
            },
            &context,
            pathsToWatch,
            FSEventStreamEventId(kFSEventStreamEventIdSinceNow),
            0.5, // Latency in seconds
            flags
        ) else {
            throw MonitorError.failedToCreateStream
        }

        self.eventStream = stream

        // Schedule stream on run loop
        FSEventStreamScheduleWithRunLoop(
            stream,
            CFRunLoopGetCurrent(),
            CFRunLoopMode.defaultMode.rawValue
        )

        // Start the stream
        guard FSEventStreamStart(stream) else {
            throw MonitorError.failedToStartStream
        }

        isMonitoring = true
        print("✓ Monitoring started for: \(path)\n")

        // Keep the run loop running
        CFRunLoopRun()
    }

    /// Stops monitoring
    func stop() {
        guard isMonitoring, let stream = eventStream else { return }

        FSEventStreamStop(stream)
        FSEventStreamInvalidate(stream)
        FSEventStreamRelease(stream)

        debounceTimer?.invalidate()
        debounceTimer = nil

        isMonitoring = false
        eventStream = nil

        print("✓ Monitoring stopped")
    }

    // MARK: - Private Methods

    /// Handles incoming file system events
    private func handleEvents(
        numEvents: Int,
        eventPaths: UnsafeMutableRawPointer,
        eventFlags: UnsafePointer<FSEventStreamEventFlags>,
        eventIds: UnsafePointer<FSEventStreamEventId>
    ) {
        let paths = Unmanaged<CFArray>.fromOpaque(eventPaths).takeUnretainedValue() as! [String]

        for i in 0..<numEvents {
            let path = paths[i]
            let flags = eventFlags[i]
            let eventId = eventIds[i]

            // Filter for projectData file changes
            if shouldProcessEvent(path: path, flags: flags) {
                handleProjectDataChange(path: path, eventId: eventId)
            }
        }
    }

    /// Determines if an event should be processed
    private func shouldProcessEvent(path: String, flags: FSEventStreamEventFlags) -> Bool {
        // Check if it's the projectData file
        if path.contains("projectData") {
            return true
        }

        // Check if it's in tracked directories
        let trackedDirs = ["Alternatives", "Resources"]
        for dir in trackedDirs {
            if path.contains("/\(dir)/") {
                return true
            }
        }

        // Ignore volatile directories
        let ignoredDirs = ["Bounces", "Freeze Files", "Autosave", ".DS_Store"]
        for dir in ignoredDirs {
            if path.contains("/\(dir)/") {
                return false
            }
        }

        return false
    }

    /// Handles a change to the projectData file or tracked directories
    private func handleProjectDataChange(path: String, eventId: FSEventStreamEventId) {
        let now = Date()

        // Log the event
        let timestamp = DateFormatter.localizedString(from: now, dateStyle: .none, timeStyle: .medium)
        print("[\(timestamp)] Event detected: \(path.components(separatedBy: "/").last ?? path)")

        // Update last event time
        lastEventTime = now

        // Reset or start debounce timer
        debounceTimer?.invalidate()

        debounceTimer = Timer.scheduledTimer(withTimeInterval: debounceThreshold, repeats: false) { [weak self] _ in
            self?.onDebounceExpired()
        }
    }

    /// Called when debounce period expires (no events for N seconds)
    private func onDebounceExpired() {
        guard let lastTime = lastEventTime else { return }

        let timeSinceLastEvent = Date().timeIntervalSince(lastTime)

        print("\n⏱️  Debounce expired (no activity for \(Int(timeSinceLastEvent))s)")
        print("📝 Would trigger auto-commit here (Phase 2 feature)")
        print("   - This would run: oxenvcs-cli add --all && oxenvcs-cli commit -m 'Auto-save'\n")

        // Reset state
        lastEventTime = nil
    }
}

// MARK: - Error Types

enum MonitorError: Error {
    case failedToCreateStream
    case failedToStartStream

    var localizedDescription: String {
        switch self {
        case .failedToCreateStream:
            return "Failed to create FSEvents stream"
        case .failedToStartStream:
            return "Failed to start FSEvents stream"
        }
    }
}
