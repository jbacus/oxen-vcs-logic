#!/bin/bash
#
# Auxin Uninstaller
# Removes all Auxin components from the system
#
# Usage:
#   ./uninstall.sh [--keep-data] [--yes]
#
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
KEEP_DATA=false
AUTO_YES=false

# Print colored messages
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_header() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --keep-data)
                KEEP_DATA=true
                shift
                ;;
            --yes|-y)
                AUTO_YES=true
                shift
                ;;
            --help)
                cat << EOF
Auxin Uninstaller

Usage: $0 [OPTIONS]

Options:
  --keep-data    Keep repository data (.oxen directories)
  --yes, -y      Skip confirmation prompt
  --help         Show this help message

Examples:
  $0                  # Interactive uninstall
  $0 --yes            # Uninstall without prompts
  $0 --keep-data      # Uninstall but preserve data

EOF
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                echo "Run '$0 --help' for usage information"
                exit 1
                ;;
        esac
    done
}

# Confirm uninstallation
confirm_uninstall() {
    if [ "$AUTO_YES" = true ]; then
        return 0
    fi

    print_header "Auxin Uninstaller"
    echo "This will remove the following components:"
    echo "  • Auxin CLI (/usr/local/bin/auxin)"
    echo "  • Auxin Daemon (/usr/local/bin/auxin-daemon)"
    echo "  • Auxin Application (/Applications/Auxin.app)"
    echo "  • Auxin Server (/usr/local/bin/auxin-server)"
    echo "  • LaunchAgent configuration"
    echo "  • Configuration files"

    if [ "$KEEP_DATA" = false ]; then
        echo ""
        print_warning "Repository data (.oxen directories) will NOT be removed"
        print_info "Use --keep-data flag explicitly if you want to preserve all data"
    fi

    echo ""
    read -p "Continue with uninstallation? (y/N) " -n 1 -r
    echo ""

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Uninstallation cancelled"
        exit 0
    fi
}

# Stop and unload daemon
stop_daemon() {
    print_header "Stopping Auxin Daemon"

    local plist_path="$HOME/Library/LaunchAgents/com.auxin.daemon.plist"

    if [ -f "$plist_path" ]; then
        print_info "Stopping daemon..."

        # Try to unload with launchctl
        if launchctl list | grep -q "com.auxin.daemon"; then
            launchctl unload "$plist_path" 2>/dev/null || true
            print_success "Daemon stopped"
        else
            print_info "Daemon not running"
        fi

        # Also try the daemon's uninstall command
        if [ -x "/usr/local/bin/auxin-daemon" ]; then
            /usr/local/bin/auxin-daemon --uninstall 2>/dev/null || true
        fi
    else
        print_info "Daemon not installed"
    fi

    echo ""
}

# Stop server if running
stop_server() {
    print_header "Stopping Auxin Server"

    local server_plist="$HOME/Library/LaunchAgents/com.auxin.server.plist"

    if [ -f "$server_plist" ]; then
        print_info "Stopping server..."
        if launchctl list | grep -q "com.auxin.server"; then
            launchctl unload "$server_plist" 2>/dev/null || true
            print_success "Server stopped"
        else
            print_info "Server not running"
        fi
    fi

    # Kill any running server processes
    if pgrep -x "auxin-server" > /dev/null; then
        print_info "Killing server process..."
        pkill -x "auxin-server" || true
        sleep 1
        print_success "Server process terminated"
    fi

    echo ""
}

# Remove binaries
remove_binaries() {
    print_header "Removing Binaries"

    local removed=false

    # Remove CLI
    if [ -f "/usr/local/bin/auxin" ]; then
        print_info "Removing auxin CLI..."
        if [ -w "/usr/local/bin" ]; then
            rm -f "/usr/local/bin/auxin"
        else
            sudo rm -f "/usr/local/bin/auxin"
        fi
        print_success "CLI removed"
        removed=true
    fi

    # Remove daemon
    if [ -f "/usr/local/bin/auxin-daemon" ]; then
        print_info "Removing auxin-daemon..."
        if [ -w "/usr/local/bin" ]; then
            rm -f "/usr/local/bin/auxin-daemon"
        else
            sudo rm -f "/usr/local/bin/auxin-daemon"
        fi
        print_success "Daemon removed"
        removed=true
    fi

    # Remove server
    if [ -f "/usr/local/bin/auxin-server" ]; then
        print_info "Removing auxin-server..."
        if [ -w "/usr/local/bin" ]; then
            rm -f "/usr/local/bin/auxin-server"
        else
            sudo rm -f "/usr/local/bin/auxin-server"
        fi
        print_success "Server removed"
        removed=true
    fi

    if [ "$removed" = false ]; then
        print_info "No binaries found"
    fi

    echo ""
}

# Remove application
remove_app() {
    print_header "Removing Application"

    if [ -d "/Applications/Auxin.app" ]; then
        print_info "Removing Auxin.app..."
        rm -rf "/Applications/Auxin.app"
        print_success "Application removed"
    else
        print_info "Application not installed"
    fi

    echo ""
}

# Remove LaunchAgent plists
remove_plists() {
    print_header "Removing LaunchAgent Configurations"

    local removed=false

    # Remove daemon plist
    local daemon_plist="$HOME/Library/LaunchAgents/com.auxin.daemon.plist"
    if [ -f "$daemon_plist" ]; then
        print_info "Removing daemon plist..."
        rm -f "$daemon_plist"
        print_success "Daemon plist removed"
        removed=true
    fi

    # Remove server plist
    local server_plist="$HOME/Library/LaunchAgents/com.auxin.server.plist"
    if [ -f "$server_plist" ]; then
        print_info "Removing server plist..."
        rm -f "$server_plist"
        print_success "Server plist removed"
        removed=true
    fi

    # Remove system-level plist if installed there
    if [ -f "/Library/LaunchAgents/com.auxin.daemon.plist" ]; then
        print_info "Removing system daemon plist..."
        sudo rm -f "/Library/LaunchAgents/com.auxin.daemon.plist"
        removed=true
    fi

    if [ "$removed" = false ]; then
        print_info "No plists found"
    fi

    echo ""
}

# Remove configuration files
remove_config() {
    print_header "Removing Configuration Files"

    local removed=false

    # Remove CLI config
    if [ -d "$HOME/.auxin" ]; then
        print_info "Removing CLI configuration..."
        rm -rf "$HOME/.auxin"
        print_success "CLI config removed"
        removed=true
    fi

    # Remove server config
    if [ -d "$HOME/.config/auxin-server" ]; then
        print_info "Removing server configuration..."
        rm -rf "$HOME/.config/auxin-server"
        print_success "Server config removed"
        removed=true
    fi

    if [ "$removed" = false ]; then
        print_info "No configuration files found"
    fi

    echo ""
}

# Remove logs
remove_logs() {
    print_header "Removing Log Files"

    local removed=false

    # Remove daemon logs
    if [ -f "/tmp/com.auxin.daemon.stdout" ]; then
        rm -f /tmp/com.auxin.daemon.stdout
        removed=true
    fi

    if [ -f "/tmp/com.auxin.daemon.stderr" ]; then
        rm -f /tmp/com.auxin.daemon.stderr
        removed=true
    fi

    # Remove server logs
    if [ -f "$HOME/Library/Logs/auxin-server.log" ]; then
        rm -f "$HOME/Library/Logs/auxin-server.log"
        removed=true
    fi

    if [ -f "$HOME/Library/Logs/auxin-server-error.log" ]; then
        rm -f "$HOME/Library/Logs/auxin-server-error.log"
        removed=true
    fi

    if [ "$removed" = true ]; then
        print_success "Log files removed"
    else
        print_info "No log files found"
    fi

    echo ""
}

# Remove server data directory
remove_server_data() {
    print_header "Removing Server Data"

    if [ -d "/var/oxen/data" ]; then
        print_info "Removing server data directory..."
        if [ -w "/var/oxen" ]; then
            rm -rf "/var/oxen/data"
        else
            sudo rm -rf "/var/oxen/data"
        fi

        # Remove parent directory if empty
        if [ -d "/var/oxen" ] && [ -z "$(ls -A /var/oxen)" ]; then
            if [ -w "/var/oxen" ]; then
                rmdir "/var/oxen"
            else
                sudo rmdir "/var/oxen"
            fi
        fi

        print_success "Server data removed"
    else
        print_info "No server data found"
    fi

    echo ""
}

# Remove shell completions
remove_completions() {
    print_header "Removing Shell Completions"

    local removed=false

    # Bash completions
    if [ -f "/usr/local/etc/bash_completion.d/auxin" ]; then
        rm -f "/usr/local/etc/bash_completion.d/auxin"
        removed=true
    fi

    if [ -f "/opt/homebrew/etc/bash_completion.d/auxin" ]; then
        rm -f "/opt/homebrew/etc/bash_completion.d/auxin"
        removed=true
    fi

    if [ -f "$HOME/.local/share/bash-completion/completions/auxin" ]; then
        rm -f "$HOME/.local/share/bash-completion/completions/auxin"
        removed=true
    fi

    # Zsh completions
    if [ -f "$HOME/.zsh/completions/_auxin" ]; then
        rm -f "$HOME/.zsh/completions/_auxin"
        removed=true
    fi

    # Fish completions
    if [ -f "$HOME/.config/fish/completions/auxin.fish" ]; then
        rm -f "$HOME/.config/fish/completions/auxin.fish"
        removed=true
    fi

    if [ "$removed" = true ]; then
        print_success "Shell completions removed"
    else
        print_info "No shell completions found"
    fi

    echo ""
}

# Print summary
print_summary() {
    print_header "Uninstallation Complete"

    print_success "Auxin has been removed from your system"
    echo ""

    if [ "$KEEP_DATA" = true ]; then
        print_info "Repository data preserved (.oxen directories)"
        print_info "To remove repository data manually, delete .oxen folders in your projects"
    else
        print_warning "Repository data (.oxen directories) were preserved"
        print_info "To remove repository data, manually delete .oxen folders in your projects"
    fi

    echo ""
    print_info "Thank you for using Auxin!"
    echo ""
}

# Main uninstallation flow
main() {
    parse_args "$@"
    confirm_uninstall

    stop_daemon
    stop_server
    remove_binaries
    remove_app
    remove_plists
    remove_config
    remove_logs
    remove_server_data
    remove_completions

    print_summary
}

# Run main function
main "$@"
