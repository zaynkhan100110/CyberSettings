\
#!/usr/bin/env bash
set -Eeuo pipefail

APP="cybersettings"
ALIAS="cse"
VERSION="0.1.0"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
BIN_SOURCE="$SCRIPT_DIR/target/release/$APP"
MANIFEST_NAME=".cybersettings-install-manifest"

say() { printf '\n[CyberSettings] %s\n' "$*"; }
die() { printf '\n[CyberSettings] ERROR: %s\n' "$*" >&2; exit 1; }
ask() {
    local prompt="$1" answer
    while true; do
        read -r -p "$prompt [y/n]: " answer
        case "$answer" in y|Y|yes|YES) return 0;; n|N|no|NO) return 1;; esac
        printf 'Please answer y or n.\n'
    done
}

[[ -x "$BIN_SOURCE" ]] || die "Release binary not found at $BIN_SOURCE. Build it first with: cargo build --release"
command -v install >/dev/null || die "'install' utility is required."

say "CyberSettings $VERSION installer"
printf 'This installer does not modify other Cyber* projects, SSH keys, GPG keys, or shell configuration.\n'
printf 'It installs only the CyberSettings executable names you select.\n\n'

scope=""
while [[ "$scope" != "user" && "$scope" != "system" ]]; do
    printf 'Installation scope:\n  1) Current user (recommended)\n  2) System-wide (all users; requires sudo)\n'
    read -r -p 'Choose [1/2]: ' choice
    case "$choice" in
        1) scope="user";;
        2) scope="system";;
        *) printf 'Choose 1 or 2.\n';;
    esac
done

if [[ "$scope" == "user" ]]; then
    DEST="$HOME/.local/bin"
    LIBDIR="$HOME/.local/lib/cybersettings/$VERSION"
    MANIFEST="$HOME/$MANIFEST_NAME"
else
    [[ "$EUID" -eq 0 ]] || die "For system-wide installation, rerun with: sudo $0"
    DEST="/usr/local/bin"
    LIBDIR="/usr/local/lib/cybersettings/$VERSION"
    MANIFEST="/var/lib/cybersettings/$MANIFEST_NAME"
fi

mkdir -p "$DEST" "$LIBDIR"
INSTALLED_BINARY="$LIBDIR/cybersettings"
install -m 0755 "$BIN_SOURCE" "$INSTALLED_BINARY"

declare -A existing=()
for name in "$APP" "$ALIAS"; do
    path="$DEST/$name"
    resolved="$(command -v "$name" 2>/dev/null || true)"
    if [[ -n "$resolved" && "$resolved" != "$path" ]]; then
        say "NOTICE: '$name' is already resolvable from PATH at: $resolved"
        printf 'The selected install directory may take precedence depending on PATH order.
'
        if ! ask "Continue considering '$name' for this installation?"; then
            existing["$name"]="skip"
        fi
    fi
    if [[ -e "$path" || -L "$path" ]] && [[ ! -e "$path" || "$(readlink -f "$path" 2>/dev/null || true)" != "$INSTALLED_BINARY" ]]; then
        existing["$name"]="$path"
        say "NOTICE: '$name' already exists at $path."
        ls -ld -- "$path" || true
        printf 'It will not be overwritten automatically.\n'
    fi
done

printf '\nChoose command names to install:\n'
printf '  1) cybersettings only\n  2) cse only\n  3) both\n'
while true; do
    read -r -p 'Choose [1/2/3]: ' names_choice
    [[ "$names_choice" == 1 || "$names_choice" == 2 || "$names_choice" == 3 ]] && break
    printf 'Choose 1, 2, or 3.\n'
done

selected=()
case "$names_choice" in
    1) selected=("$APP");;
    2) selected=("$ALIAS");;
    3) selected=("$APP" "$ALIAS");;
esac

installed=()
for name in "${selected[@]}"; do
    path="$DEST/$name"
    if [[ "${existing[$name]:-}" == "skip" ]]; then
        say "Skipping '$name' because an existing PATH command was detected."
        continue
    fi
    if [[ -n "${existing[$name]:-}" ]]; then
        if [[ -d "$path" && ! -L "$path" ]]; then
            say "Skipping '$name': the destination is a directory, and the installer will never remove directories."
            continue
        fi
        if ask "Replace the existing '$name' at $path?"; then
            rm -f -- "$path"
        else
            say "Skipping '$name'."
            continue
        fi
    fi
    ln -sfn -- "$INSTALLED_BINARY" "$path"
    installed+=("$name")
done

if [[ "$scope" == "system" ]]; then
    mkdir -p "$(dirname "$MANIFEST")"
fi
{
    printf 'scope=%s\n' "$scope"
    printf 'destination=%s\n' "$DEST"
    printf 'source=%s\n' "$BIN_SOURCE"
    printf 'binary=%s\n' "$INSTALLED_BINARY"
    for name in "${installed[@]}"; do printf 'command=%s\n' "$name"; done
} > "$MANIFEST"

if [[ "$scope" == "user" && ":$PATH:" != *":$DEST:"* ]]; then
    say "$DEST is not currently in PATH."
    printf 'Add this line to your shell profile (for example ~/.bashrc or ~/.zshrc):\n'
    printf '  export PATH="$HOME/.local/bin:$PATH"\n'
fi

say "Installation finished."
printf 'Installed command(s): %s\n' "${installed[*]:-(none)}"
printf 'Installed binary: %s\n' "$INSTALLED_BINARY"
printf 'Test with: %s --version\n' "${installed[0]:-$APP}"
