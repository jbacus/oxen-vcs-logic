import AppKit

// Create and configure the application
let app = NSApplication.shared
app.setActivationPolicy(.regular)

// Set up the app delegate
let delegate = AppDelegate()
app.delegate = delegate

// Activate the app and run
app.activate(ignoringOtherApps: true)
app.run()
