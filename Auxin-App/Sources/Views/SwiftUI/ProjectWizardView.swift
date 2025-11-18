import SwiftUI
import UniformTypeIdentifiers

struct ProjectWizardView: View {
    @Environment(\.dismiss) private var dismiss
    @State private var projectPath: String = ""
    @State private var detectedType: ProjectType?
    @State private var isInitializing = false
    @State private var statusMessage: String = ""
    @State private var showingFilePicker = false
    @State private var showingError = false
    @State private var showingSuccess = false
    @State private var errorMessage: String = ""
    @State private var successMessage: String = ""

    var body: some View {
        VStack(spacing: 20) {
            // Header
            VStack(alignment: .leading, spacing: 8) {
                Text("Initialize Creative Project")
                    .font(.title2)
                    .fontWeight(.semibold)

                Text("Select a project to initialize with version control. This will create a new Oxen repository and start monitoring the project for changes.")
                    .font(.body)
                    .foregroundColor(.secondary)
                    .fixedSize(horizontal: false, vertical: true)

                // Supported types
                HStack(spacing: 16) {
                    ForEach(ProjectType.allCases, id: \.self) { type in
                        HStack(spacing: 4) {
                            Image(systemName: type.iconName)
                                .foregroundColor(.secondary)
                            Text(".\(type.fileExtension)")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }
                }
                .padding(.top, 4)
            }
            .padding(.top)

            // Project Path Selection
            VStack(alignment: .leading, spacing: 8) {
                Text("Project Path:")
                    .font(.headline)

                HStack(spacing: 12) {
                    TextField("Select a creative project...", text: $projectPath)
                        .textFieldStyle(.roundedBorder)
                        .disabled(true)

                    Button(action: {
                        showingFilePicker = true
                    }) {
                        Text("Browse...")
                            .frame(width: 100)
                    }
                    .buttonStyle(.borderedProminent)
                }

                // Show detected project type
                if let type = detectedType {
                    HStack(spacing: 6) {
                        Image(systemName: type.iconName)
                            .foregroundColor(.accentColor)
                        Text("Detected: \(type.displayName)")
                            .font(.subheadline)
                            .foregroundColor(.accentColor)
                    }
                    .padding(.top, 4)
                }
            }

            // Status Message
            if !statusMessage.isEmpty {
                HStack {
                    if isInitializing {
                        ProgressView()
                            .scaleEffect(0.8)
                            .padding(.trailing, 4)
                    }
                    Text(statusMessage)
                        .font(.subheadline)
                        .foregroundColor(isInitializing ? .secondary : .green)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.vertical, 8)
            }

            Spacer()

            // Action Buttons
            HStack {
                Spacer()

                Button("Cancel") {
                    dismiss()
                }
                .keyboardShortcut(.cancelAction)

                Button("Initialize") {
                    initializeProject()
                }
                .buttonStyle(.borderedProminent)
                .disabled(projectPath.isEmpty || isInitializing || detectedType == nil)
                .keyboardShortcut(.defaultAction)
            }
        }
        .padding(24)
        .frame(width: 600, height: 380)
        .fileImporter(
            isPresented: $showingFilePicker,
            allowedContentTypes: [.package, .folder, .item],
            allowsMultipleSelection: false
        ) { result in
            handleFileSelection(result)
        }
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

    // MARK: - Actions

    private func handleFileSelection(_ result: Result<[URL], Error>) {
        switch result {
        case .success(let urls):
            guard let url = urls.first else { return }

            // Detect project type
            if let type = ProjectType.detect(from: url.path) {
                projectPath = url.path
                detectedType = type
                statusMessage = "Ready to initialize: \(url.lastPathComponent)"
            } else {
                let supportedExtensions = ProjectType.supportedExtensions.map { ".\($0)" }.joined(separator: ", ")
                showError("Unsupported project type.\n\nSupported types: \(supportedExtensions)")
                detectedType = nil
            }

        case .failure(let error):
            showError("Failed to select file: \(error.localizedDescription)")
        }
    }

    private func initializeProject() {
        // Validate path
        guard !projectPath.isEmpty else {
            showError("Please select a project path")
            return
        }

        guard let projectType = detectedType else {
            let supportedExtensions = ProjectType.supportedExtensions.map { ".\($0)" }.joined(separator: ", ")
            showError("Unsupported project type.\n\nSupported types: \(supportedExtensions)")
            return
        }

        guard FileManager.default.fileExists(atPath: projectPath) else {
            showError("Selected project does not exist")
            return
        }

        // Show progress
        isInitializing = true
        statusMessage = "Initializing \(projectType.displayName) repository..."

        // Initialize project via XPC
        OxenDaemonXPCClient.shared.initializeProject(path: projectPath) { success, error in
            DispatchQueue.main.async {
                isInitializing = false
                statusMessage = ""

                if success {
                    // Check if project was already initialized
                    if error == "already_initialized" {
                        successMessage = "Project already initialized!\n\nThis \(projectType.displayName) project was previously initialized (possibly via CLI). It has been registered for monitoring.\n\nAutomatic commits will be created after 30 seconds of inactivity."
                    } else {
                        successMessage = "\(projectType.displayName) project initialized successfully!\n\nThe project is now being monitored for changes. Automatic commits will be created after 30 seconds of inactivity."
                    }
                    showingSuccess = true

                    // Post notification to refresh project list
                    NotificationCenter.default.post(name: .refreshProjects, object: nil)
                } else {
                    let errorMsg = error ?? "Unknown error occurred"
                    errorMessage = "Failed to initialize project:\n\n\(errorMsg)\n\nMake sure the auxin CLI tool is installed at /usr/local/bin/auxin"
                    showingError = true
                }
            }
        }
    }

    private func showError(_ message: String) {
        errorMessage = message
        showingError = true
    }
}

// MARK: - Preview

struct ProjectWizardView_Previews: PreviewProvider {
    static var previews: some View {
        ProjectWizardView()
    }
}
