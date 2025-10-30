import SwiftUI
import UniformTypeIdentifiers

struct ProjectWizardView: View {
    @Environment(\.dismiss) private var dismiss
    @State private var projectPath: String = ""
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
                Text("Initialize Logic Pro Project")
                    .font(.title2)
                    .fontWeight(.semibold)

                Text("Select a Logic Pro project (.logicx) to initialize with version control. This will create a new Oxen repository and start monitoring the project for changes.")
                    .font(.body)
                    .foregroundColor(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
            .padding(.top)

            // Project Path Selection
            VStack(alignment: .leading, spacing: 8) {
                Text("Project Path:")
                    .font(.headline)

                HStack(spacing: 12) {
                    TextField("Select a Logic Pro project...", text: $projectPath)
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
                .disabled(projectPath.isEmpty || isInitializing)
                .keyboardShortcut(.defaultAction)
            }
        }
        .padding(24)
        .frame(width: 600, height: 350)
        .fileImporter(
            isPresented: $showingFilePicker,
            allowedContentTypes: [.package, .folder],
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

            // Validate .logicx extension
            if url.pathExtension == "logicx" {
                projectPath = url.path
                statusMessage = "Ready to initialize: \(url.lastPathComponent)"
            } else {
                showError("Please select a valid Logic Pro project (.logicx)")
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

        guard projectPath.hasSuffix(".logicx") else {
            showError("Selected file must be a Logic Pro project (.logicx)")
            return
        }

        guard FileManager.default.fileExists(atPath: projectPath) else {
            showError("Selected project does not exist")
            return
        }

        // Show progress
        isInitializing = true
        statusMessage = "Initializing Oxen repository..."

        // Initialize project via XPC
        OxenDaemonXPCClient.shared.initializeProject(path: projectPath) { success, error in
            DispatchQueue.main.async {
                isInitializing = false
                statusMessage = ""

                if success {
                    // Check if project was already initialized
                    if error == "already_initialized" {
                        successMessage = "Project already initialized!\n\nThis project was previously initialized (possibly via CLI). It has been registered for monitoring.\n\nAutomatic commits will be created after 30 seconds of inactivity."
                    } else {
                        successMessage = "Project initialized successfully!\n\nThe project is now being monitored for changes. Automatic commits will be created after 30 seconds of inactivity."
                    }
                    showingSuccess = true

                    // Post notification to refresh project list
                    NotificationCenter.default.post(name: .refreshProjects, object: nil)
                } else {
                    let errorMsg = error ?? "Unknown error occurred"
                    errorMessage = "Failed to initialize project:\n\n\(errorMsg)\n\nMake sure the oxenvcs-cli tool is installed at /usr/local/bin/oxenvcs-cli"
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
