import SwiftUI

struct MilestoneCommitView: View {
    let project: Project
    @Environment(\.dismiss) private var dismiss

    @State private var message: String = ""
    @State private var bpm: String = ""
    @State private var sampleRate: String = ""
    @State private var keySignature: String = ""
    @State private var timeSignature: String = ""
    @State private var tags: String = ""
    @State private var cleanupTemporaryFiles: Bool = true
    @State private var isCommitting: Bool = false
    @State private var showingError: Bool = false
    @State private var showingSuccess: Bool = false
    @State private var errorMessage: String = ""
    @State private var successMessage: String = ""

    var body: some View {
        VStack(spacing: 20) {
            // Header
            VStack(alignment: .leading, spacing: 8) {
                Text("Create Milestone Commit")
                    .font(.title2)
                    .fontWeight(.semibold)

                Text("Create a milestone commit with metadata for \(project.displayName)")
                    .font(.body)
                    .foregroundColor(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
            .padding(.top)

            // Form
            Form {
                Section {
                    TextField("e.g., Verse 1 completed", text: $message)
                        .textFieldStyle(.roundedBorder)
                } header: {
                    Text("Commit Message")
                        .font(.headline)
                }

                Section {
                    HStack {
                        Text("BPM:")
                            .frame(width: 120, alignment: .leading)
                        TextField("120", text: $bpm)
                            .textFieldStyle(.roundedBorder)
                            .frame(width: 100)
                    }

                    HStack {
                        Text("Sample Rate:")
                            .frame(width: 120, alignment: .leading)
                        TextField("44100", text: $sampleRate)
                            .textFieldStyle(.roundedBorder)
                            .frame(width: 100)
                    }

                    HStack {
                        Text("Key Signature:")
                            .frame(width: 120, alignment: .leading)
                        TextField("C Major", text: $keySignature)
                            .textFieldStyle(.roundedBorder)
                            .frame(width: 150)
                    }

                    HStack {
                        Text("Time Signature:")
                            .frame(width: 120, alignment: .leading)
                        TextField("4/4", text: $timeSignature)
                            .textFieldStyle(.roundedBorder)
                            .frame(width: 100)
                    }
                } header: {
                    Text("Project Metadata")
                        .font(.headline)
                }

                Section {
                    TextField("verse, vocals, production", text: $tags)
                        .textFieldStyle(.roundedBorder)
                } header: {
                    Text("Tags (comma-separated)")
                        .font(.headline)
                }

                Section {
                    Toggle("Clean up temporary files (Bounces, Freeze Files)", isOn: $cleanupTemporaryFiles)
                } header: {
                    Text("Options")
                        .font(.headline)
                }
            }
            .formStyle(.grouped)

            // Status Message
            if isCommitting {
                HStack {
                    ProgressView()
                        .scaleEffect(0.8)
                        .padding(.trailing, 4)
                    Text("Creating milestone commit...")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }

            Spacer()

            // Action Buttons
            HStack {
                Spacer()

                Button("Cancel") {
                    dismiss()
                }
                .keyboardShortcut(.cancelAction)

                Button("Commit") {
                    createMilestoneCommit()
                }
                .buttonStyle(.borderedProminent)
                .disabled(message.isEmpty || isCommitting)
                .keyboardShortcut(.defaultAction)
            }
        }
        .padding(24)
        .frame(width: 600, height: 550)
        .alert("Error", isPresented: $showingError) {
            Button("OK", role: .cancel) { }
        } message: {
            Text(errorMessage)
        }
        .alert("Success", isPresented: $showingSuccess) {
            Button("OK", role: .cancel) {
                dismiss()
            }
        } message: {
            Text(successMessage)
        }
    }

    private func createMilestoneCommit() {
        // Validate message
        guard !message.isEmpty else {
            showError("Please enter a commit message")
            return
        }

        isCommitting = true

        // Perform cleanup if requested
        if cleanupTemporaryFiles {
            performCleanup()
        }

        // Build metadata
        var metadata: [String: Any] = [:]

        if let bpmValue = Double(bpm) {
            metadata["bpm"] = bpmValue
        }

        if let sampleRateValue = Int(sampleRate) {
            metadata["sample_rate"] = sampleRateValue
        }

        if !keySignature.isEmpty {
            metadata["key_signature"] = keySignature
        }

        if !timeSignature.isEmpty {
            metadata["time_signature"] = timeSignature
        }

        if !tags.isEmpty {
            let tagList = tags.split(separator: ",").map { $0.trimmingCharacters(in: .whitespaces) }
            metadata["tags"] = tagList
        }

        // Create commit via XPC
        OxenDaemonXPCClient.shared.commitProject(path: project.path, message: message, metadata: metadata) { success in
            DispatchQueue.main.async {
                isCommitting = false

                if success {
                    successMessage = "Milestone commit created successfully!"
                    showingSuccess = true

                    // Post notification to refresh project list
                    NotificationCenter.default.post(name: .refreshProjects, object: nil)
                } else {
                    errorMessage = "Failed to create milestone commit. Please check the logs for details."
                    showingError = true
                }
            }
        }
    }

    private func performCleanup() {
        let foldersToClean = ["Bounces", "Freeze Files", "Media.localized"]

        for folder in foldersToClean {
            let folderPath = (project.path as NSString).appendingPathComponent(folder)
            try? FileManager.default.removeItem(atPath: folderPath)
        }
    }

    private func showError(_ message: String) {
        errorMessage = message
        showingError = true
    }
}
