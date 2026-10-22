
#!/usr/bin/env bash
set -Eeuo pipefail

APP="xerokey"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
CARGO_BIN="${HOME}/.cargo/bin"
OS="$(uname -s)"
ARCH="$(uname -m)"
SOURCE=""
TMP=""

info() { printf '\033[1;36m[INFO]\033[0m %s\n' "$*"; }
ok()   { printf '\033[1;32m[ OK ]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[WARN]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[ERROR]\033[0m %s\n' "$*" >&2; exit 1; }

cleanup() {
    if [[ -n "$TMP" && -e "$TMP" ]]; then
        rm -f -- "$TMP"
    fi
}
trap cleanup EXIT

[[ -f "$SCRIPT_DIR/Cargo.toml" ]] ||
    die "Cargo.toml not found. Run this installer from a valid XeroKey project."

# Detect Termux.
IS_TERMUX=false
if [[ -n "${PREFIX:-}" && "${PREFIX}" == *com.termux* ]] ||
   [[ -d /data/data/com.termux/files/usr ]]; then
    IS_TERMUX=true
fi

# Determine the target platform and installation directory.
if [[ "$IS_TERMUX" == true ]]; then
    [[ "$ARCH" == "aarch64" ]] ||
        die "The bundled Android binary supports ARM64 only. Detected: $ARCH"

    TARGET="aarch64-linux-android"
    INSTALL_DIR="${PREFIX:-/data/data/com.termux/files/usr}/bin"
elif [[ "$OS" == "Linux" ]]; then
    INSTALL_DIR="${HOME}/.local/bin"

    case "$ARCH" in
        aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
        x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
        *)
            die "Unsupported Linux architecture: $ARCH"
            ;;
    esac
else
    die "Unsupported operating system: $OS"
fi

mkdir -p "$INSTALL_DIR"
[[ -w "$INSTALL_DIR" ]] ||
    die "Cannot write to installation directory: $INSTALL_DIR"

# Put the standard rustup installation directory on PATH if present.
if [[ -d "$CARGO_BIN" ]]; then
    export PATH="$CARGO_BIN:$PATH"
fi

# Check whether a candidate binary can actually run on this device.
# XeroKey currently exits with code 2 for --help, so inspect its output.
is_compatible_binary() {
    local binary="$1"
    local output=""

    [[ -f "$binary" && -x "$binary" ]] || return 1

    output="$("$binary" --help 2>&1)" || true

    [[ "$output" == *"XeroKey"* && "$output" == *"Usage:"* ]]
}

# Select a bundled executable for the detected platform.
find_compatible_binary() {
    local candidate
    local candidates=()

    if [[ "$IS_TERMUX" == true ]]; then
        candidates+=("$SCRIPT_DIR/bin/xerokey-aarch64-linux-android")
    else
        candidates+=("$SCRIPT_DIR/bin/xerokey-$TARGET")
    fi

    for candidate in "${candidates[@]}"; do
        if is_compatible_binary "$candidate"; then
            printf '%s\n' "$candidate"
            return 0
        fi
    done

    return 1
}

# Prefer the prebuilt binary, avoiding unnecessary compilation.
if SOURCE="$(find_compatible_binary)"; then
    info "Compatible bundled binary found for $TARGET."
    info "Skipping compilation."
else
    info "No compatible bundled binary found for $TARGET."

    # Build from source when Cargo and Rust are already installed.
    if command -v cargo >/dev/null 2>&1 &&
       command -v rustc >/dev/null 2>&1; then

        info "Building XeroKey from source."

        (
            cd "$SCRIPT_DIR"
            cargo build --release --locked
        )

        BUILT="$SCRIPT_DIR/target/release/$APP"

        [[ -f "$BUILT" && -x "$BUILT" ]] ||
            die "Build succeeded but the executable is missing."

        if ! is_compatible_binary "$BUILT"; then
            die "The built executable failed its compatibility check."
        fi

        SOURCE="$BUILT"

    elif [[ "$IS_TERMUX" == true ]]; then
        die "No compatible Android binary and Rust/Cargo is unavailable. Install Rust with: pkg install rust"

    else
        # Only offer rustup on regular Linux, with explicit consent.
        [[ -t 0 ]] ||
            die "Rust is missing. Install it from https://rustup.rs/ and rerun this script."

        command -v curl >/dev/null 2>&1 ||
            die "curl is required to download the official Rust installer."

        printf 'Rust/Cargo is missing. Install Rust using official rustup? [y/N] '
        read -r answer

        case "$answer" in
            y|Y|yes|YES) ;;
            *) die "Cancelled. Install Rust and rerun the installer." ;;
        esac

        info "Installing Rust using the official rustup installer."

        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
            sh -s -- -y --profile minimal

        export PATH="$CARGO_BIN:$PATH"

        command -v cargo >/dev/null 2>&1 ||
            die "Cargo was not found after installing Rust."

        command -v rustc >/dev/null 2>&1 ||
            die "rustc was not found after installing Rust."

        info "Building XeroKey from source."

        (
            cd "$SCRIPT_DIR"
            cargo build --release --locked
        )

        BUILT="$SCRIPT_DIR/target/release/$APP"

        [[ -f "$BUILT" && -x "$BUILT" ]] ||
            die "Build succeeded but the executable is missing."

        if ! is_compatible_binary "$BUILT"; then
            die "The built executable failed its compatibility check."
        fi

        SOURCE="$BUILT"
    fi
fi

# Stage the executable in the destination directory.
TMP="$(mktemp "$INSTALL_DIR/.${APP}.tmp.XXXXXX")"
cp -- "$SOURCE" "$TMP"
chmod 755 "$TMP"

[[ -s "$TMP" ]] || die "The staged executable is empty."

# Verify the staged copy before replacing an existing installation.
if ! is_compatible_binary "$TMP"; then
    die "The staged executable failed validation; installation aborted."
fi

mv -f -- "$TMP" "$INSTALL_DIR/$APP"
TMP=""

ok "XeroKey installed to $INSTALL_DIR/$APP"

case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        warn "$INSTALL_DIR is not currently in PATH."
        if [[ "$IS_TERMUX" == true ]]; then
            printf 'Try: export PATH="%s:$PATH"\n' "$INSTALL_DIR"
        else
            printf 'Add this to your shell profile: export PATH="%s:$PATH"\n' "$INSTALL_DIR"
        fi
        ;;
esac

printf '\nTest the installation with:\n  xerokey --help\n'
