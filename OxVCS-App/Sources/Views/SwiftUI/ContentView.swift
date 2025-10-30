import SwiftUI

struct ContentView: View {
    @StateObject private var viewModel = ProjectListViewModel()
    @State private var selectedProject: Project?
    @State private var showingMergeHelper = false

    var body: some View {
        NavigationSplitView {
            // Left sidebar - Project List
            ProjectListContentView(
                projects: viewModel.projects,
                selectedProject: $selectedProject
            )
            .navigationSplitViewColumnWidth(min: 250, ideal: 300, max: 400)
        } detail: {
            // Right panel - Project Details
            if let project = selectedProject {
                ProjectDetailContentView(project: project)
            } else {
                VStack {
                    Spacer()
                    Text("Select a project to view details")
                        .foregroundColor(.secondary)
                        .font(.title3)
                    Spacer()
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .toolbar {
            ToolbarItemGroup(placement: .automatic) {
                Button(action: {
                    if let appDelegate = NSApplication.shared.delegate as? AppDelegate {
                        appDelegate.showProjectWizard()
                    }
                }) {
                    Label("Add Project", systemImage: "plus.circle")
                }

                Button(action: {
                    viewModel.loadProjects()
                }) {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
            }
        }
        .sheet(isPresented: $showingMergeHelper) {
            if let project = selectedProject {
                MergeHelperView(project: project)
            }
        }
        .onAppear {
            viewModel.loadProjects()
        }
        .onReceive(NotificationCenter.default.publisher(for: .refreshProjects)) { _ in
            viewModel.loadProjects()
        }
        .onReceive(NotificationCenter.default.publisher(for: .showMergeHelper)) { _ in
            if selectedProject != nil {
                showingMergeHelper = true
            } else {
                showNoProjectAlert()
            }
        }
        .overlay(alignment: .bottom) {
            SwiftUIStatusBar()
        }
    }

    private func showNoProjectAlert() {
        let alert = NSAlert()
        alert.messageText = "No Project Selected"
        alert.informativeText = "Please select a project first"
        alert.alertStyle = .informational
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }
}
