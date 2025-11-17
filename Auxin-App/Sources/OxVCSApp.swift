import SwiftUI

@main
struct AuxinApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate

    var body: some Scene {
        WindowGroup {
            ContentView()
                .frame(minWidth: 800, minHeight: 600)
        }
        .windowStyle(.automatic)
        .commands {
            CommandGroup(replacing: .appInfo) {
                Button("About Auxin") {
                    appDelegate.showAbout()
                }
            }
            CommandGroup(replacing: .newItem) {
                Button("Initialize New Project...") {
                    appDelegate.showProjectWizard()
                }
                .keyboardShortcut("n", modifiers: .command)
            }
            CommandMenu("View") {
                Button("Refresh Project List") {
                    NotificationCenter.default.post(name: .refreshProjects, object: nil)
                }
                .keyboardShortcut("r", modifiers: .command)

                Divider()

                Button("Merge Helper...") {
                    appDelegate.showMergeHelperFromMenu()
                }
            }
        }
    }
}

// Notification names
extension Notification.Name {
    static let refreshProjects = Notification.Name("refreshProjects")
    static let showMergeHelper = Notification.Name("showMergeHelper")
    static let showProjectWizard = Notification.Name("showProjectWizard")
}
