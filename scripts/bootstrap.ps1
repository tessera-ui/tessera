# Tessera bootstrap script for Windows
#
# This script ensures that the required Rust toolchain is installed.

# --- Helper Functions ---

# Function to print a message in a consistent format
function Print-Message {
    param (
        [string]$Message
    )
    Write-Host "--- $Message ---"
}

# Function to check if a command exists
function Command-Exists {
    param (
        [string]$Command
    )
    return (Get-Command $Command -ErrorAction SilentlyContinue)
}

# --- Main Logic ---

Print-Message "Starting Tessera bootstrap for Windows..."

# Check for Rust
if (Command-Exists "rustc") {
    Print-Message "Rust is already installed."
} else {
    Print-Message "Rust is not installed. We will now download and run rustup-init.exe."
    Print-Message "Please follow the on-screen instructions in the installer."

    # Define the rustup-init.exe URL and output path
    $rustupUrl = "https://win.rustup.rs/x86_64"
    $rustupPath = "$env:TEMP\rustup-init.exe"

    # Download the installer
    try {
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath
        Print-Message "Downloaded rustup-init.exe to $rustupPath"
    } catch {
        Print-Message "Error: Failed to download the Rust installer."
        Print-Message "Please download it manually from https://www.rust-lang.org/tools/install"
        exit 1
    }

    # Run the installer
    Print-Message "Launching the Rust installer..."
    # We run it with -y to perform a default installation non-interactively.
    Start-Process -FilePath $rustupPath -ArgumentList "-y" -Wait
    
    # Clean up the installer
    Remove-Item $rustupPath

    Print-Message "Rust installation process finished."
    Print-Message "You may need to restart your terminal for the 'cargo' command to be available."
}

Print-Message "Bootstrap finished. You should now be able to build the project with 'cargo build'."
