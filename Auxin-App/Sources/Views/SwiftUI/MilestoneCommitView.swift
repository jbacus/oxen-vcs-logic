import SwiftUI

struct MilestoneCommitView: View {
    let project: Project
    @Environment(\.dismiss) private var dismiss

    // Common fields
    @State private var message: String = ""
    @State private var tags: String = ""
    @State private var cleanupTemporaryFiles: Bool = true

    // Logic Pro metadata
    @State private var bpm: String = ""
    @State private var sampleRate: String = ""
    @State private var keySignature: String = ""
    @State private var timeSignature: String = ""

    // SketchUp metadata
    @State private var units: String = ""
    @State private var layerCount: String = ""
    @State private var componentCount: String = ""
    @State private var groupCount: String = ""

    // Blender metadata
    @State private var sceneCount: String = ""
    @State private var objectCount: String = ""
    @State private var materialCount: String = ""
    @State private var renderEngine: String = "CYCLES"
    @State private var resolution: String = ""
    @State private var fps: String = ""
    @State private var frameRange: String = ""

    // UI state
    @State private var isCommitting: Bool = false
    @State private var showingError: Bool = false
    @State private var showingSuccess: Bool = false
    @State private var errorMessage: String = ""
    @State private var successMessage: String = ""

    var body: some View {
        VStack(spacing: 20) {
            // Header
            VStack(alignment: .leading, spacing: 8) {
                HStack(spacing: 8) {
                    Image(systemName: project.iconName)
                        .foregroundColor(.accentColor)
                    Text("Create Milestone Commit")
                        .font(.title2)
                        .fontWeight(.semibold)
                }

                Text("Create a milestone commit with metadata for \(project.displayName)")
                    .font(.body)
                    .foregroundColor(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
            .padding(.top)

            // Form
            Form {
                Section {
                    TextField("e.g., \(exampleMessage)", text: $message)
                        .textFieldStyle(.roundedBorder)
                } header: {
                    Text("Commit Message")
                        .font(.headline)
                }

                // Project-type-specific metadata
                metadataSection

                Section {
                    TextField("production, milestone, v1.0", text: $tags)
                        .textFieldStyle(.roundedBorder)
                } header: {
                    Text("Tags (comma-separated)")
                        .font(.headline)
                }

                Section {
                    Toggle("Clean up temporary files (\(cleanupDescription))", isOn: $cleanupTemporaryFiles)
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
        .frame(width: 600, height: frameHeight)
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

    // MARK: - Project-Type-Specific UI

    @ViewBuilder
    private var metadataSection: some View {
        Section {
            switch project.projectType {
            case .logicPro:
                logicProMetadataFields

            case .sketchup:
                sketchUpMetadataFields

            case .blender:
                blenderMetadataFields
            }
        } header: {
            Text("\(project.projectType.displayName) Metadata")
                .font(.headline)
        }
    }

    @ViewBuilder
    private var logicProMetadataFields: some View {
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
    }

    @ViewBuilder
    private var sketchUpMetadataFields: some View {
        HStack {
            Text("Units:")
                .frame(width: 120, alignment: .leading)
            Picker("", selection: $units) {
                Text("Select...").tag("")
                Text("Inches").tag("Inches")
                Text("Feet").tag("Feet")
                Text("Millimeters").tag("Millimeters")
                Text("Centimeters").tag("Centimeters")
                Text("Meters").tag("Meters")
            }
            .labelsHidden()
            .frame(width: 150)
        }

        HStack {
            Text("Layer Count:")
                .frame(width: 120, alignment: .leading)
            TextField("15", text: $layerCount)
                .textFieldStyle(.roundedBorder)
                .frame(width: 100)
        }

        HStack {
            Text("Components:")
                .frame(width: 120, alignment: .leading)
            TextField("234", text: $componentCount)
                .textFieldStyle(.roundedBorder)
                .frame(width: 100)
        }

        HStack {
            Text("Groups:")
                .frame(width: 120, alignment: .leading)
            TextField("12", text: $groupCount)
                .textFieldStyle(.roundedBorder)
                .frame(width: 100)
        }
    }

    @ViewBuilder
    private var blenderMetadataFields: some View {
        HStack {
            Text("Scenes:")
                .frame(width: 120, alignment: .leading)
            TextField("1", text: $sceneCount)
                .textFieldStyle(.roundedBorder)
                .frame(width: 100)
        }

        HStack {
            Text("Objects:")
                .frame(width: 120, alignment: .leading)
            TextField("1247", text: $objectCount)
                .textFieldStyle(.roundedBorder)
                .frame(width: 100)
        }

        HStack {
            Text("Materials:")
                .frame(width: 120, alignment: .leading)
            TextField("45", text: $materialCount)
                .textFieldStyle(.roundedBorder)
                .frame(width: 100)
        }

        HStack {
            Text("Render Engine:")
                .frame(width: 120, alignment: .leading)
            Picker("", selection: $renderEngine) {
                Text("Cycles").tag("CYCLES")
                Text("Eevee").tag("EEVEE")
                Text("Workbench").tag("WORKBENCH")
            }
            .labelsHidden()
            .frame(width: 150)
        }

        HStack {
            Text("Resolution:")
                .frame(width: 120, alignment: .leading)
            TextField("1920x1080", text: $resolution)
                .textFieldStyle(.roundedBorder)
                .frame(width: 150)
        }

        HStack {
            Text("FPS:")
                .frame(width: 120, alignment: .leading)
            TextField("30", text: $fps)
                .textFieldStyle(.roundedBorder)
                .frame(width: 100)
        }

        HStack {
            Text("Frame Range:")
                .frame(width: 120, alignment: .leading)
            TextField("1-240", text: $frameRange)
                .textFieldStyle(.roundedBorder)
                .frame(width: 150)
        }
    }

    // MARK: - Computed Properties

    private var exampleMessage: String {
        switch project.projectType {
        case .logicPro:
            return "Verse 1 completed"
        case .sketchup:
            return "Added kitchen cabinets"
        case .blender:
            return "Rigging completed"
        }
    }

    private var cleanupDescription: String {
        project.projectType.cleanupFolders.joined(separator: ", ")
    }

    private var frameHeight: CGFloat {
        switch project.projectType {
        case .logicPro:
            return 550
        case .sketchup:
            return 550
        case .blender:
            return 700  // Blender has more fields
        }
    }

    // MARK: - Actions

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

        // Build metadata based on project type
        var metadata: [String: Any] = [:]

        switch project.projectType {
        case .logicPro:
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

        case .sketchup:
            if !units.isEmpty {
                metadata["units"] = units
            }
            if let layerValue = Int(layerCount) {
                metadata["layer_count"] = layerValue
            }
            if let componentValue = Int(componentCount) {
                metadata["component_count"] = componentValue
            }
            if let groupValue = Int(groupCount) {
                metadata["group_count"] = groupValue
            }

        case .blender:
            if let sceneValue = Int(sceneCount) {
                metadata["scene_count"] = sceneValue
            }
            if let objectValue = Int(objectCount) {
                metadata["object_count"] = objectValue
            }
            if let materialValue = Int(materialCount) {
                metadata["material_count"] = materialValue
            }
            if !renderEngine.isEmpty {
                metadata["render_engine"] = renderEngine
            }
            if !resolution.isEmpty {
                metadata["resolution"] = resolution
            }
            if let fpsValue = Int(fps) {
                metadata["fps"] = fpsValue
            }
            if !frameRange.isEmpty {
                metadata["frame_range"] = frameRange
            }
        }

        // Add common metadata
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
        for folder in project.projectType.cleanupFolders {
            let folderPath = (project.path as NSString).appendingPathComponent(folder)
            try? FileManager.default.removeItem(atPath: folderPath)
        }
    }

    private func showError(_ message: String) {
        errorMessage = message
        showingError = true
    }
}
