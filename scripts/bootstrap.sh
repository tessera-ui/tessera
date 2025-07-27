#!/bin/bash
# Tessera bootstrap script for POSIX systems

# --- Helper Functions ---

# Function to print a message in a consistent format
print_message() {
    echo "--- $1 ---"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install packages using a specific package manager
install_packages() {
    local pm=$1
    shift
    local packages=("$@")

    print_message "Updating package list with $pm..."
    case $pm in
        apt)
            sudo apt update
            ;;
        pacman)
            sudo pacman -Sy
            ;;
        brew)
            brew update
            ;;
    esac

    print_message "Installing dependencies: ${packages[*]}..."
    if ! sudo "$pm" install -y "${packages[@]}"; then
        print_message "Error: Failed to install packages with $pm. Please install them manually."
        exit 1
    fi
}

# --- Main Logic ---

print_message "Starting Tessera bootstrap..."

# --- Rust Installation ---
if ! command_exists rustc; then
    print_message "Rust is not installed. Installing via rustup..."
    if ! curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; then
        print_message "Error: Failed to install Rust. Please install it manually from https://rustup.rs/."
        exit 1
    fi
    # Add cargo to the current shell's PATH
    source "$HOME/.cargo/env"
else
    print_message "Rust is already installed."
fi



# --- OS-Specific Dependencies ---
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    print_message "Detected Linux OS."

    # Define packages for different display servers and package managers
    declare -A deps
    deps["apt,x11"]="libx11-dev libxrandr-dev libxcursor-dev"
    deps["apt,wayland"]="libwayland-dev libxkbcommon-dev"
    deps["pacman,x11"]="libx11 libxrandr libxcursor"
    deps["pacman,wayland"]="wayland libxkbcommon"

    pm=""
    if command_exists apt; then
        pm="apt"
    elif command_exists pacman; then
        pm="pacman"
    fi

    if [ -n "$pm" ]; then
        session_type=$(echo "$XDG_SESSION_TYPE" | tr '[:upper:]' '[:lower:]')
        
        if [[ "$session_type" == "x11" ]]; then
            print_message "Detected X11 session. Installing X11 dependencies."
            install_packages "$pm" ${deps["$pm,x11"]}
        elif [[ "$session_type" == "wayland" ]]; then
            print_message "Detected Wayland session. Installing Wayland dependencies."
            install_packages "$pm" ${deps["$pm,wayland"]}
        else
            print_message "Could not automatically detect display server. Please choose which dependencies to install."
            select choice in "X11" "Wayland" "Both" "Skip"; do
                case $choice in
                    X11)
                        install_packages "$pm" ${deps["$pm,x11"]}
                        break
                        ;;
                    Wayland)
                        install_packages "$pm" ${deps["$pm,wayland"]}
                        break
                        ;;
                    Both)
                        install_packages "$pm" ${deps["$pm,x11"]} ${deps["$pm,wayland"]}
                        break
                        ;;
                    Skip)
                        print_message "Skipping display server dependency installation."
                        break
                        ;;
                    *) echo "Invalid option $REPLY";;
                esac
            done
        fi
    else
        print_message "Unsupported Linux distribution. Please install the equivalent of the following packages manually:"
        echo " - For X11: libx11, libxrandr, libxcursor (and their -dev/-devel packages)"
        echo " - For Wayland: wayland, libxkbcommon (and their -dev/-devel packages)"
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    print_message "Detected macOS."
    print_message "No additional system packages are required for macOS."
else
    print_message "Unsupported OS: $OSTYPE. Please install dependencies manually."
fi

print_message "Bootstrap finished. You should now be able to build the project with 'cargo run'."
