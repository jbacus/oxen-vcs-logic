import SwiftUI

struct SwiftUIStatusBar: View {
    @State private var daemonState: DaemonState = .checking
    @State private var projectCount: Int = 0
    @State private var timer: Timer?
    @State private var rotation: Double = 0
    @State private var isHovering: Bool = false
    @State private var rotationTimer: Timer?

    var body: some View {
        HStack {
            // Daemon status - Interactive button
            Button(action: {
                restartDaemon()
            }) {
                HStack(spacing: 6) {
                    // Status indicator or restart icon
                    ZStack {
                        if isHovering {
                            // Show restart icon on hover
                            Image(systemName: "arrow.counterclockwise.circle.fill")
                                .foregroundColor(.orange)
                                .font(.system(size: 12))
                        } else if daemonState.needsProgressIndicator {
                            // Animated progress indicator for transient states
                            Circle()
                                .stroke(daemonState.color.opacity(0.3), lineWidth: 2)
                                .frame(width: 12, height: 12)

                            Circle()
                                .trim(from: 0, to: 0.75)
                                .stroke(daemonState.color, style: StrokeStyle(lineWidth: 2.5, lineCap: .round))
                                .frame(width: 14, height: 14)
                                .rotationEffect(.degrees(rotation))
                        } else {
                            // Solid circle for stable states
                            Circle()
                                .fill(daemonState.color)
                                .frame(width: 8, height: 8)
                        }
                    }
                    .frame(width: 14, height: 14)

                    Text("Daemon: \(daemonState.displayText)")
                        .font(daemonState == .restarting ? .caption.bold() : .caption)
                        .foregroundColor(daemonState == .restarting ? daemonState.color : .primary)
                        .animation(.easeInOut(duration: 0.3), value: daemonState)
                }
            }
            .buttonStyle(PlainButtonStyle())
            .onHover { hovering in
                withAnimation(.easeInOut(duration: 0.2)) {
                    isHovering = hovering
                }
            }
            .help("Click to restart daemon")

            Spacer()

            // Project count
            Text("\(projectCount) projects monitored")
                .font(.caption)
                .foregroundColor(.secondary)

            Spacer()

            // Version number
            if let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String {
                Text("v\(version)")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(Color(nsColor: .controlBackgroundColor))
        .onAppear {
            updateStatus()
            startTimer()
        }
        .onDisappear {
            stopTimer()
        }
        .onReceive(NotificationCenter.default.publisher(for: .daemonStateChanged)) { notification in
            if let newState = notification.object as? DaemonState {
                DispatchQueue.main.async {
                    daemonState = newState

                    // Start or stop rotation animation based on state
                    if newState.needsProgressIndicator {
                        startRotationAnimation()
                    } else {
                        stopRotationAnimation()
                    }

                    // Auto-refresh after restarting state
                    if newState == .restarting {
                        // Wait a few seconds then check status
                        DispatchQueue.main.asyncAfter(deadline: .now() + 3.0) {
                            updateStatus()
                        }
                    }
                }
            }
        }
    }

    private func updateStatus() {
        // Don't override if we're in a manual state transition
        guard daemonState != .restarting else { return }

        OxenDaemonXPCClient.shared.ping { isRunning in
            DispatchQueue.main.async {
                if daemonState == .restarting {
                    // Don't override restarting state
                    return
                }

                if isRunning {
                    daemonState = .running
                } else {
                    daemonState = .stopped
                }
            }
        }

        // Update project count
        OxenDaemonXPCClient.shared.getMonitoredProjects { projects in
            DispatchQueue.main.async {
                projectCount = projects.count
            }
        }
    }

    private func startTimer() {
        timer = Timer.scheduledTimer(withTimeInterval: 5.0, repeats: true) { _ in
            updateStatus()
        }
    }

    private func stopTimer() {
        timer?.invalidate()
        timer = nil
    }

    private func restartDaemon() {
        // Write to log file for debugging
        let log = "[\(Date())] StatusBar - restartDaemon function called\n"
        try? log.write(toFile: "/tmp/auxin-restart-button.log", atomically: false, encoding: .utf8)

        print("DEBUG: StatusBar - restartDaemon function called")

        // Update the daemon state to restarting - this triggers the status indicator animation
        DispatchQueue.main.async {
            NotificationCenter.default.post(name: .daemonStateChanged, object: DaemonState.restarting)
        }

        OxenDaemonXPCClient.shared.restartDaemon { success, error in
            print("DEBUG: StatusBar - Got callback from XPC client: success=\(success), error=\(error ?? "nil")")

            // Append callback to log
            let log2 = "[\(Date())] StatusBar - Got callback: success=\(success), error=\(error ?? "nil")\n"
            if let handle = FileHandle(forWritingAtPath: "/tmp/auxin-restart-button.log") {
                handle.seekToEndOfFile()
                handle.write(log2.data(using: .utf8)!)
                handle.closeFile()
            }

            DispatchQueue.main.async {
                if success {
                    // Reset state to checking so updateStatus can detect the new daemon
                    DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) {
                        NotificationCenter.default.post(name: .daemonStateChanged, object: DaemonState.checking)
                        // Immediately check status to update to running
                        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                            self.updateStatus()
                        }
                    }
                } else {
                    // Reset status back to checking on failure
                    NotificationCenter.default.post(name: .daemonStateChanged, object: DaemonState.checking)
                }
            }
        }
    }

    private func startRotationAnimation() {
        // Stop any existing rotation timer
        rotationTimer?.invalidate()

        // Reset rotation to 0
        rotation = 0

        // Start a timer that updates rotation continuously
        rotationTimer = Timer.scheduledTimer(withTimeInterval: 0.02, repeats: true) { _ in
            rotation += 9 // 360 degrees / (0.8 seconds / 0.02 interval) = 9 degrees per tick
            if rotation >= 360 {
                rotation -= 360
            }
        }
    }

    private func stopRotationAnimation() {
        rotationTimer?.invalidate()
        rotationTimer = nil
        rotation = 0
    }
}
