# How to Install Auxin

This document outlines the different ways to install Auxin, depending on your needs.

- **For Most Users (GUI + CLI):** [Use the DMG Installer](#for-most-users-dmg-installer)
- **For Command-Line Users:** [Use Homebrew](#for-command-line-users-homebrew)
- **For Developers (Build from Source):** [Use the Build Script](#for-developers-build-from-source)

---

## For Most Users (DMG Installer)

This is the recommended method for most users. It provides a standard macOS installation experience for both the graphical application and the command-line tools.

1.  **Download the latest version:**
    - Go to the [Auxin GitHub Releases](https://github.com/jbacus/auxin/releases) page.
    - Download the `Auxin-vX.Y.Z.dmg` file (where `X.Y.Z` is the latest version).

2.  **Install the application:**
    - Open the downloaded `.dmg` file.
    - In the window that appears, drag the `Auxin.app` icon into your `Applications` folder.

3.  **Install the command-line tools and background service:**
    - Launch the `Auxin` application from your `Applications` folder.
    - On first launch, you will be prompted to install the helper tools (CLI and daemon). Follow the on-screen instructions.

4.  **Enable the background service:**
    - Open `System Settings` > `General` > `Login Items`.
    - Find `Auxin Daemon` in the "Allow in the Background" section and enable it.

---

## For Command-Line Users (Homebrew)

If you only need the `auxin` command-line interface (CLI) and do not need the graphical application, you can install it via [Homebrew](https://brew.sh/).

```bash
# First, tap the official Auxin repository
brew tap jbacus/auxin

# Then, install the CLI
brew install auxin
```

This will install the `auxin` binary and all required shell completions.

**Note:** This method does *not* install the background daemon for automatic draft commits.

---

## For Developers (Build from Source)

If you are a developer who wants to contribute to Auxin or build it from the source code, you can use the provided build script.

### Prerequisites

- **macOS 14.0+**
- **Xcode Command Line Tools:** `xcode-select --install`
- **Rust:** `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Oxen CLI:** `pip3 install oxen-ai`

### Installation Script

The root of this repository contains a comprehensive `install.sh` script that builds and installs all components:

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/jbacus/auxin.git
    cd auxin
    ```

2.  **Run the installer:**
    ```bash
    ./install.sh
    ```

This script will:
- Check for all prerequisites.
- Build the Rust CLI (`auxin`).
- Build the Swift daemon (`auxin-daemon`).
- Build the Swift GUI (`Auxin.app`).
- Install all binaries to `/usr/local/bin`.
- Install the application to `/Applications`.
- Install and register the Launch Agent for the background daemon.
- Install shell completions for the CLI.

To see all available options, run `./install.sh --help`.

### Uninstalling

To uninstall all components installed by the script, run:
```bash
./install.sh --uninstall
```