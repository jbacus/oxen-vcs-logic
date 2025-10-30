import SwiftUI

struct SwiftUIStatusBar: View {
    @State private var daemonStatus: String = "Checking..."
    @State private var daemonColor: Color = .gray
    @State private var projectCount: Int = 0
    @State private var timer: Timer?

    var body: some View {
        HStack {
            // Daemon status
            HStack(spacing: 4) {
                Circle()
                    .fill(daemonColor)
                    .frame(width: 8, height: 8)
                Text("Daemon: \(daemonStatus)")
                    .font(.caption)
            }

            Spacer()

            // Project count
            Text("\(projectCount) projects monitored")
                .font(.caption)
                .foregroundColor(.secondary)
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
    }

    private func updateStatus() {
        OxenDaemonXPCClient.shared.ping { isRunning in
            DispatchQueue.main.async {
                if isRunning {
                    daemonStatus = "Running"
                    daemonColor = .green
                } else {
                    daemonStatus = "Not Running"
                    daemonColor = .red
                }
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
}
