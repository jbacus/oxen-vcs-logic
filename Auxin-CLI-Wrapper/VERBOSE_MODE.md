# Verbose Mode Documentation

The `auxin` tool includes a comprehensive `--verbose` (or `-v`) mode that provides detailed debugging output to help troubleshoot issues with Logic Pro project detection and initialization.

## Usage

Add the `--verbose` or `-v` flag to any command:

```bash
auxin --verbose <command>
auxin -v <command>
```

## Examples

### Initialize with verbose output

```bash
# From inside a Logic Pro project
cd Demo_Project.logicx
auxin --verbose init --logic .

# From parent directory
auxin -v init --logic ./Demo_Project.logicx
```

### Add files with verbose output

```bash
auxin --verbose add --all
```

## What Verbose Mode Shows

### 1. Logic Pro Project Detection

When using `--verbose`, you'll see detailed information about the detection process:

```
[DEBUG] === Logic Pro Project Detection ===
[DEBUG] Input path: ./Demo_Project.logicx
[DEBUG] Checking if path exists...
[DEBUG] ✓ Path exists
[DEBUG] Checking if path is a directory...
[DEBUG] ✓ Path is a directory
[DEBUG] Canonicalizing path to resolve relative paths...
[DEBUG] Canonical path: /Users/you/Music/Logic/Demo_Project.logicx
[DEBUG] Checking for .logicx extension...
[DEBUG] Found extension: 'logicx'
[DEBUG] ✓ Valid .logicx extension
[DEBUG] Searching for ProjectData file...
```

### 2. ProjectData File Search

The tool shows you exactly where it's looking for the ProjectData file:

```
[DEBUG] --- Searching for ProjectData file ---
[DEBUG] Checking for Alternatives directory: /Users/you/Music/Logic/Demo_Project.logicx/Alternatives
[DEBUG] ✓ Alternatives directory exists
[DEBUG] Scanning subdirectories in Alternatives/...
[DEBUG]   Checking subdirectory: 004
[DEBUG]     Looking for: /Users/you/Music/Logic/Demo_Project.logicx/Alternatives/004/ProjectData
[DEBUG]     ✓ Found ProjectData!
[INFO] Successfully detected Logic Pro project: /Users/you/Music/Logic/Demo_Project.logicx
[DEBUG] ProjectData location: /Users/you/Music/Logic/Demo_Project.logicx/Alternatives/004/ProjectData
```

### 3. Initialization Steps

See each step of the initialization process:

```
[DEBUG] === Initializing Logic Pro Project Repository ===
[DEBUG] Target path: ./Demo_Project.logicx
[DEBUG] Step 1: Detecting Logic Pro project structure...
[INFO] Detected Logic Pro project: Demo_Project
[DEBUG] Project name: Demo_Project
[DEBUG] Step 2: Initializing Oxen repository...
[INFO] Initialized Oxen repository at: ./Demo_Project.logicx
[DEBUG] Step 3: Creating .oxenignore file...
[DEBUG] Ignore file path: /Users/you/Music/Logic/Demo_Project.logicx/.oxenignore
[DEBUG] Generated ignore patterns (342 bytes)
[INFO] Created .oxenignore file
[DEBUG] Step 4: Initializing draft branch workflow...
[INFO] Initializing draft branch workflow...
[DEBUG] Draft branch initialized successfully
[DEBUG] === Initialization Complete ===
✓ Successfully initialized Logic Pro project repository
```

## Log Levels

The tool uses different log levels with color coding:

- **[DEBUG]** (Blue) - Detailed step-by-step information (only shown with `--verbose`)
- **[INFO]** (Green) - Important progress messages (always shown)
- **[WARN]** (Yellow) - Warnings about potential issues
- **[ERROR]** (Red) - Error messages
- **✓** (Green) - Success messages

## Troubleshooting Examples

### Problem: Can't find ProjectData file

Run with verbose mode to see where the tool is looking:

```bash
$ auxin -v init --logic ./Demo_Project.logicx

[DEBUG] === Logic Pro Project Detection ===
[DEBUG] Input path: ./Demo_Project.logicx
[DEBUG] ✓ Path exists
[DEBUG] ✓ Path is a directory
[DEBUG] Canonical path: /Users/you/Music/Logic/Demo_Project.logicx
[DEBUG] ✓ Valid .logicx extension
[DEBUG] --- Searching for ProjectData file ---
[DEBUG] Checking for Alternatives directory: /Users/you/Music/Logic/Demo_Project.logicx/Alternatives
[DEBUG] ❌ Alternatives directory does not exist
[DEBUG] Checking for ProjectData at root level...
[DEBUG]   Checking: /Users/you/Music/Logic/Demo_Project.logicx/ProjectData
[DEBUG]   ❌ Not found: ProjectData
[DEBUG]   Checking: /Users/you/Music/Logic/Demo_Project.logicx/projectData
[DEBUG]   ❌ Not found: projectData
[DEBUG]   Checking: /Users/you/Music/Logic/Demo_Project.logicx/Project Data
[DEBUG]   ❌ Not found: Project Data
[DEBUG] ❌ No ProjectData file found in any expected location

Error: Failed to detect Logic Pro project
Caused by:
    No ProjectData file found in /Users/you/Music/Logic/Demo_Project.logicx
```

This shows you that:
1. The Alternatives directory doesn't exist
2. The tool checked for ProjectData at the root in multiple case variations
3. None of the expected files were found

**Solution**: This might not be a valid Logic Pro project, or it hasn't been saved yet in Logic Pro. Open the project in Logic Pro and save it to create the project structure.

### Problem: Wrong extension detected

```bash
$ auxin -v init --logic ./MyProject

[DEBUG] === Logic Pro Project Detection ===
[DEBUG] Input path: ./MyProject
[DEBUG] ✓ Path exists
[DEBUG] ✓ Path is a directory
[DEBUG] Canonical path: /Users/you/Music/MyProject
[DEBUG] Checking for .logicx extension...
[DEBUG] Found extension: ''
[DEBUG] ❌ Extension is not 'logicx'

Error: Failed to detect Logic Pro project
Caused by:
    Path is not a Logic Pro folder project (.logicx): /Users/you/Music/MyProject
```

**Solution**: Make sure you're pointing to a `.logicx` directory, not a regular folder.

## Tips

1. **Always use verbose mode when troubleshooting** - It shows exactly what the tool is checking
2. **Look for checkmarks (✓) vs crosses (❌)** - They quickly show what passed or failed
3. **Check the canonical path** - This shows the absolute path the tool is actually checking
4. **Note subdirectory names** - The tool shows which subdirectories it finds in Alternatives/

## Common Issues and Solutions

| Symptom | Verbose Output Shows | Solution |
|---------|---------------------|----------|
| "No ProjectData found" | ❌ Alternatives directory does not exist | Open project in Logic Pro and save it |
| "Path is not a .logicx" | Found extension: '' | Point to the .logicx directory, not parent |
| "Path does not exist" | Input path shown incorrectly | Check your path spelling and relative vs absolute |
| Can't find in subdirectory | Shows wrong subdirectory number | Check if Logic has multiple alternatives |

## Reporting Bugs

When reporting issues, please include:
1. The full command you ran with `--verbose`
2. The complete verbose output
3. Your Logic Pro project structure: `ls -la path/to/project.logicx/`

This helps maintainers diagnose issues quickly!
