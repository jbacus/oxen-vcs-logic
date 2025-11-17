import AppKit

/// Status bar view for displaying daemon status and project information
class StatusBarView: NSView {
    private let statusLabel = NSTextField(labelWithString: "")
    private let projectCountLabel = NSTextField(labelWithString: "")
    private let separator = NSBox()

    override init(frame frameRect: NSRect) {
        super.init(frame: frameRect)
        setupUI()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setupUI() {
        wantsLayer = true
        layer?.backgroundColor = NSColor.controlBackgroundColor.cgColor

        // Status label (left side)
        statusLabel.font = NSFont.systemFont(ofSize: 11)
        statusLabel.textColor = .secondaryLabelColor
        statusLabel.translatesAutoresizingMaskIntoConstraints = false
        addSubview(statusLabel)

        // Separator
        separator.boxType = .separator
        separator.translatesAutoresizingMaskIntoConstraints = false
        addSubview(separator)

        // Project count label (right side)
        projectCountLabel.font = NSFont.systemFont(ofSize: 11)
        projectCountLabel.textColor = .secondaryLabelColor
        projectCountLabel.alignment = .right
        projectCountLabel.translatesAutoresizingMaskIntoConstraints = false
        addSubview(projectCountLabel)

        NSLayoutConstraint.activate([
            // Status label on left
            statusLabel.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 8),
            statusLabel.centerYAnchor.constraint(equalTo: centerYAnchor),

            // Separator at top
            separator.topAnchor.constraint(equalTo: topAnchor),
            separator.leadingAnchor.constraint(equalTo: leadingAnchor),
            separator.trailingAnchor.constraint(equalTo: trailingAnchor),
            separator.heightAnchor.constraint(equalToConstant: 1),

            // Project count on right
            projectCountLabel.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -8),
            projectCountLabel.centerYAnchor.constraint(equalTo: centerYAnchor),
            projectCountLabel.widthAnchor.constraint(greaterThanOrEqualToConstant: 100)
        ])

        // Initial state
        updateStatus(message: "Connecting...", color: .secondaryLabelColor)
        updateProjectCount(0)
    }

    func updateStatus(message: String, color: NSColor) {
        statusLabel.stringValue = message
        statusLabel.textColor = color
    }

    func updateProjectCount(_ count: Int) {
        let plural = count == 1 ? "project" : "projects"
        projectCountLabel.stringValue = "\(count) \(plural) monitored"
    }
}
