# OxVCS UI Implementation Plan

## Overview

This document outlines the step-by-step implementation plan for building the UX mockups defined in `UI_MOCKUPS.md`.

## Phase 1: Foundation & Design System (Week 1)

### 1.1 Create Design System Components

**File**: `Sources/DesignSystem/OxVCSDesignSystem.swift`

```swift
// Color palette
enum OxVCSColor {
    static let background = NSColor(hex: "1e1e1e")
    static let panel = NSColor(hex: "252526")
    static let border = NSColor(hex: "3e3e42")
    static let textPrimary = NSColor(hex: "cccccc")
    static let textSecondary = NSColor(hex: "858585")
    static let accent = NSColor(hex: "007acc")
    static let success = NSColor(hex: "4ec9b0")
    static let warning = NSColor(hex: "ff9800")
    static let error = NSColor(hex: "f44747")
}

// Typography
enum OxVCSFont {
    static let headerLarge = NSFont.systemFont(ofSize: 20, weight: .semibold)
    static let header = NSFont.systemFont(ofSize: 18, weight: .semibold)
    static let body = NSFont.systemFont(ofSize: 13, weight: .regular)
    static let metadata = NSFont.systemFont(ofSize: 11, weight: .medium)
    static let mono = NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)
}

// Spacing
enum OxVCSSpacing {
    static let margin: CGFloat = 16
    static let paddingSmall: CGFloat = 12
    static let paddingMedium: CGFloat = 16
    static let paddingLarge: CGFloat = 24
    static let gapTight: CGFloat = 8
    static let gapStandard: CGFloat = 12
    static let gapLoose: CGFloat = 16
}
```

### 1.2 Create Reusable UI Components

**File**: `Sources/Components/OxVCSButton.swift`

```swift
class OxVCSButton: NSButton {
    enum Style {
        case primary
        case secondary
        case danger
    }

    init(title: String, style: Style = .primary) {
        super.init(frame: .zero)
        self.title = title
        self.bezelStyle = .rounded
        applyStyle(style)
    }

    private func applyStyle(_ style: Style) {
        switch style {
        case .primary:
            self.contentTintColor = OxVCSColor.accent
        case .secondary:
            self.contentTintColor = OxVCSColor.textSecondary
        case .danger:
            self.contentTintColor = OxVCSColor.error
        }
    }
}
```

**File**: `Sources/Components/OxVCSCard.swift`

```swift
class OxVCSCard: NSView {
    init() {
        super.init(frame: .zero)
        wantsLayer = true
        layer?.backgroundColor = OxVCSColor.panel.cgColor
        layer?.cornerRadius = 8
        layer?.borderWidth = 1
        layer?.borderColor = OxVCSColor.border.cgColor
    }
}
```

**File**: `Sources/Components/OxVCSMetadataLabel.swift`

```swift
class OxVCSMetadataLabel: NSStackView {
    init(bpm: Int?, sampleRate: Int?, key: String?) {
        super.init(frame: .zero)
        orientation = .horizontal
        spacing = OxVCSSpacing.gapStandard

        if let bpm = bpm {
            addArrangedSubview(createLabel("BPM: \(bpm)"))
        }
        if let sr = sampleRate {
            addArrangedSubview(createLabel("\(sr/1000)kHz"))
        }
        if let key = key {
            addArrangedSubview(createLabel(key))
        }
    }

    private func createLabel(_ text: String) -> NSTextField {
        let label = NSTextField(labelWithString: text)
        label.font = OxVCSFont.metadata
        label.textColor = OxVCSColor.textSecondary
        return label
    }
}
```

## Phase 2: Redesign Main Window (Week 2)

### 2.1 Rebuild MainViewController

**File**: `Sources/Views/MainViewController.swift` (complete rewrite)

```swift
class MainViewController: NSViewController {
    // MARK: - Properties
    private var viewModel = ProjectListViewModel()
    private var cancellables = Set<AnyCancellable>()

    // Left sidebar
    private let projectBrowserView = ProjectBrowserView()

    // Right panel
    private let projectDetailView = ProjectDetailPanelView()

    // Split view
    private let splitView = NSSplitView()

    override func loadView() {
        view = NSView(frame: NSRect(x: 0, y: 0, width: 1200, height: 800))
        view.wantsLayer = true
        view.layer?.backgroundColor = OxVCSColor.background.cgColor
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        setupSplitView()
        bindViewModel()
    }

    private func setupSplitView() {
        splitView.isVertical = true
        splitView.dividerStyle = .thin

        // Left sidebar: 300pt fixed
        let sidebarContainer = NSView()
        sidebarContainer.addSubview(projectBrowserView)
        projectBrowserView.translatesAutoresizingMaskIntoConstraints = false
        NSLayoutConstraint.activate([
            projectBrowserView.topAnchor.constraint(equalTo: sidebarContainer.topAnchor),
            projectBrowserView.leadingAnchor.constraint(equalTo: sidebarContainer.leadingAnchor),
            projectBrowserView.trailingAnchor.constraint(equalTo: sidebarContainer.trailingAnchor),
            projectBrowserView.bottomAnchor.constraint(equalTo: sidebarContainer.bottomAnchor),
            sidebarContainer.widthAnchor.constraint(equalToConstant: 300)
        ])

        splitView.addArrangedSubview(sidebarContainer)
        splitView.addArrangedSubview(projectDetailView)

        view.addSubview(splitView)
        splitView.translatesAutoresizingMaskIntoConstraints = false
        NSLayoutConstraint.activate([
            splitView.topAnchor.constraint(equalTo: view.topAnchor),
            splitView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            splitView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            splitView.bottomAnchor.constraint(equalTo: view.bottomAnchor)
        ])
    }
}
```

### 2.2 Create Project Browser Sidebar

**File**: `Sources/Views/ProjectBrowserView.swift`

```swift
class ProjectBrowserView: NSView {
    private let headerLabel = NSTextField(labelWithString: "PROJECTS")
    private let tableView = NSTableView()
    private let scrollView = NSScrollView()
    private let addButton = OxVCSButton(title: "+ Add Project", style: .secondary)
    private let refreshButton = OxVCSButton(title: "⟲ Refresh", style: .secondary)

    private var projects: [Project] = []

    override init(frame: NSRect) {
        super.init(frame: frame)
        setupUI()
    }

    private func setupUI() {
        wantsLayer = true
        layer?.backgroundColor = OxVCSColor.panel.cgColor

        // Header
        headerLabel.font = OxVCSFont.metadata
        headerLabel.textColor = OxVCSColor.textSecondary
        addSubview(headerLabel)

        // Table view
        tableView.style = .plain
        tableView.backgroundColor = .clear
        tableView.dataSource = self
        tableView.delegate = self

        let column = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("project"))
        column.title = ""
        tableView.addTableColumn(column)
        tableView.headerView = nil

        scrollView.documentView = tableView
        scrollView.hasVerticalScroller = true
        addSubview(scrollView)

        // Buttons
        addSubview(addButton)
        addSubview(refreshButton)

        setupConstraints()
    }

    private func setupConstraints() {
        [headerLabel, scrollView, addButton, refreshButton].forEach {
            $0.translatesAutoresizingMaskIntoConstraints = false
        }

        NSLayoutConstraint.activate([
            headerLabel.topAnchor.constraint(equalTo: topAnchor, constant: OxVCSSpacing.margin),
            headerLabel.leadingAnchor.constraint(equalTo: leadingAnchor, constant: OxVCSSpacing.margin),

            scrollView.topAnchor.constraint(equalTo: headerLabel.bottomAnchor, constant: OxVCSSpacing.gapStandard),
            scrollView.leadingAnchor.constraint(equalTo: leadingAnchor),
            scrollView.trailingAnchor.constraint(equalTo: trailingAnchor),
            scrollView.bottomAnchor.constraint(equalTo: addButton.topAnchor, constant: -OxVCSSpacing.gapStandard),

            addButton.leadingAnchor.constraint(equalTo: leadingAnchor, constant: OxVCSSpacing.margin),
            addButton.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -OxVCSSpacing.margin),
            addButton.bottomAnchor.constraint(equalTo: refreshButton.topAnchor, constant: -OxVCSSpacing.gapTight),
            addButton.heightAnchor.constraint(equalToConstant: 32),

            refreshButton.leadingAnchor.constraint(equalTo: leadingAnchor, constant: OxVCSSpacing.margin),
            refreshButton.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -OxVCSSpacing.margin),
            refreshButton.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -OxVCSSpacing.margin),
            refreshButton.heightAnchor.constraint(equalToConstant: 28)
        ])
    }
}

extension ProjectBrowserView: NSTableViewDataSource {
    func numberOfRows(in tableView: NSTableView) -> Int {
        return projects.count
    }
}

extension ProjectBrowserView: NSTableViewDelegate {
    func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
        let cellView = ProjectRowView(project: projects[row])
        return cellView
    }

    func tableView(_ tableView: NSTableView, heightOfRow row: Int) -> CGFloat {
        return 72
    }
}
```

### 2.3 Create Project Row Cell

**File**: `Sources/Views/ProjectRowView.swift`

```swift
class ProjectRowView: NSView {
    private let iconLabel = NSTextField(labelWithString: "🎵")
    private let nameLabel = NSTextField(labelWithString: "")
    private let statusLabel = NSTextField(labelWithString: "")
    private let commitCountLabel = NSTextField(labelWithString: "")
    private let lockIcon = NSImageView()
    private let statusDot = NSView()

    init(project: Project) {
        super.init(frame: .zero)
        configure(with: project)
        setupUI()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setupUI() {
        // Icon
        iconLabel.font = NSFont.systemFont(ofSize: 24)
        addSubview(iconLabel)

        // Name
        nameLabel.font = OxVCSFont.body
        nameLabel.textColor = OxVCSColor.textPrimary
        nameLabel.lineBreakMode = .byTruncatingTail
        addSubview(nameLabel)

        // Status
        statusLabel.font = OxVCSFont.metadata
        statusLabel.textColor = OxVCSColor.textSecondary
        addSubview(statusLabel)

        // Commit count
        commitCountLabel.font = OxVCSFont.metadata
        commitCountLabel.textColor = OxVCSColor.textSecondary
        addSubview(commitCountLabel)

        // Status dot
        statusDot.wantsLayer = true
        statusDot.layer?.cornerRadius = 4
        addSubview(statusDot)

        // Lock icon
        lockIcon.image = NSImage(systemSymbolName: "lock.fill", accessibilityDescription: "Locked")
        lockIcon.contentTintColor = OxVCSColor.warning
        lockIcon.isHidden = true
        addSubview(lockIcon)

        setupConstraints()
    }

    private func configure(with project: Project) {
        nameLabel.stringValue = project.displayName
        commitCountLabel.stringValue = "\(project.commitCount) commits"

        if let lastCommit = project.lastCommit {
            let formatter = RelativeDateTimeFormatter()
            formatter.unitsStyle = .abbreviated
            statusLabel.stringValue = "Updated \(formatter.localizedString(for: lastCommit, relativeTo: Date()))"
        }

        // Status dot color
        if project.isLocked {
            statusDot.layer?.backgroundColor = OxVCSColor.warning.cgColor
            lockIcon.isHidden = false
        } else {
            statusDot.layer?.backgroundColor = OxVCSColor.success.cgColor
        }
    }

    private func setupConstraints() {
        [iconLabel, nameLabel, statusLabel, commitCountLabel, statusDot, lockIcon].forEach {
            $0.translatesAutoresizingMaskIntoConstraints = false
        }

        NSLayoutConstraint.activate([
            iconLabel.leadingAnchor.constraint(equalTo: leadingAnchor, constant: OxVCSSpacing.paddingSmall),
            iconLabel.centerYAnchor.constraint(equalTo: centerYAnchor),

            statusDot.leadingAnchor.constraint(equalTo: iconLabel.trailingAnchor, constant: OxVCSSpacing.gapTight),
            statusDot.topAnchor.constraint(equalTo: topAnchor, constant: OxVCSSpacing.paddingSmall + 2),
            statusDot.widthAnchor.constraint(equalToConstant: 8),
            statusDot.heightAnchor.constraint(equalToConstant: 8),

            nameLabel.leadingAnchor.constraint(equalTo: statusDot.trailingAnchor, constant: OxVCSSpacing.gapTight),
            nameLabel.topAnchor.constraint(equalTo: topAnchor, constant: OxVCSSpacing.paddingSmall),
            nameLabel.trailingAnchor.constraint(equalTo: lockIcon.leadingAnchor, constant: -OxVCSSpacing.gapTight),

            lockIcon.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -OxVCSSpacing.paddingSmall),
            lockIcon.centerYAnchor.constraint(equalTo: nameLabel.centerYAnchor),
            lockIcon.widthAnchor.constraint(equalToConstant: 16),
            lockIcon.heightAnchor.constraint(equalToConstant: 16),

            commitCountLabel.leadingAnchor.constraint(equalTo: nameLabel.leadingAnchor),
            commitCountLabel.topAnchor.constraint(equalTo: nameLabel.bottomAnchor, constant: 2),

            statusLabel.leadingAnchor.constraint(equalTo: nameLabel.leadingAnchor),
            statusLabel.topAnchor.constraint(equalTo: commitCountLabel.bottomAnchor, constant: 2)
        ])
    }
}
```

## Phase 3: Rebuild Commit Dialog (Week 3)

### 3.1 Complete Redesign of MilestoneCommitWindow

**File**: `Sources/Views/MilestoneCommitWindow.swift` (rewrite)

Key improvements:
- Use design system colors and fonts
- Add file list with checkboxes
- Add real-time validation
- Add smart auto-complete for metadata
- Add cleanup options with clear descriptions
- Add keyboard shortcuts

## Phase 4: Rebuild History Viewer (Week 4)

### 4.1 Create Timeline-Based History View

**File**: `Sources/Views/HistoryTimelineView.swift`

Features:
- Grouped by date (Today, Yesterday, Last Week, etc.)
- Rich commit cards with metadata
- Filtering by tags, date, author
- Search functionality
- Quick actions (restore, tag, view changes)

## Phase 5: Enhanced Status & Lock Management (Week 5)

### 5.1 Redesign Status View

**File**: `Sources/Views/StatusChangesView.swift`

Features:
- Three sections: Staged, Modified, Untracked
- Individual file controls
- File size display
- Ignored files section (collapsible)

### 5.2 Redesign Lock Management

**File**: `Sources/Views/LockManagementView.swift`

Features:
- Visual status indicators
- Preset duration options
- Lock history
- Team notifications (future)

## Phase 6: Polish & Animations (Week 6)

### 6.1 Add Smooth Transitions

```swift
// Fade in/out
NSAnimationContext.runAnimationGroup { context in
    context.duration = 0.2
    view.animator().alphaValue = 0.0
}

// Slide in
NSAnimationContext.runAnimationGroup { context in
    context.duration = 0.3
    context.timingFunction = CAMediaTimingFunction(name: .easeInEaseOut)
    view.animator().frame.origin.y = finalY
}
```

### 6.2 Add Loading States

```swift
class OxVCSLoadingIndicator: NSProgressIndicator {
    init() {
        super.init(frame: NSRect(x: 0, y: 0, width: 32, height: 32))
        style = .spinning
        controlTint = .defaultControlTint
    }
}
```

### 6.3 Add Empty States

```swift
class EmptyStateView: NSView {
    init(icon: String, message: String, action: String?) {
        super.init(frame: .zero)

        let iconLabel = NSTextField(labelWithString: icon)
        iconLabel.font = NSFont.systemFont(ofSize: 48)

        let messageLabel = NSTextField(wrappingLabelWithString: message)
        messageLabel.font = OxVCSFont.body
        messageLabel.textColor = OxVCSColor.textSecondary
        messageLabel.alignment = .center

        // Layout stack view
        let stackView = NSStackView(views: [iconLabel, messageLabel])
        stackView.orientation = .vertical
        stackView.spacing = OxVCSSpacing.gapStandard
        stackView.alignment = .centerX

        addSubview(stackView)
        // ... constraints
    }
}
```

## Testing Plan

### Unit Tests
- Design system color conversions
- Metadata label formatting
- Project cell configuration

### Integration Tests
- Main window layout
- Split view resizing
- Data binding

### UI Tests
- Keyboard shortcuts
- Dialog flows
- Error states

## Performance Considerations

1. **Table View Optimization**
   - Use `NSTableCellView` reuse
   - Lazy load commit history
   - Virtual scrolling for large lists

2. **Image Caching**
   - Cache status icons
   - Preload common symbols

3. **Async Operations**
   - Load project details asynchronously
   - Show loading indicators
   - Cancel in-flight requests on navigation

## Accessibility

1. **VoiceOver Support**
   - All buttons have accessibility labels
   - Table cells have meaningful descriptions
   - Keyboard navigation works everywhere

2. **Keyboard Shortcuts**
   - Implement all shortcuts from mockups
   - Add Help menu with shortcut list

3. **High Contrast Mode**
   - Test with system high contrast
   - Ensure sufficient color contrast ratios

## Migration Strategy

1. **Incremental Rollout**
   - Keep old views initially
   - Add feature flags for new UI
   - A/B test with users

2. **Backwards Compatibility**
   - Ensure data models unchanged
   - XPC protocol remains stable
   - Settings migrate automatically

## Documentation

- Add inline code documentation
- Create UI component library docs
- Update user guide with screenshots

---

## Implementation Order

1. ✅ Phase 1: Design System (Week 1)
2. ✅ Phase 2: Main Window (Week 2)
3. ✅ Phase 3: Commit Dialog (Week 3)
4. ✅ Phase 4: History Viewer (Week 4)
5. ✅ Phase 5: Status & Locks (Week 5)
6. ✅ Phase 6: Polish (Week 6)

**Total Timeline**: 6 weeks to production-ready UI
