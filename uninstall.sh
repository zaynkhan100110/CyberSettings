\
#!/usr/bin/env bash
set -Eeuo pipefail

APP="cybersettings"
ALIAS="cse"
MANIFEST_NAME=".cybersettings-install-manifest"

say() { printf '\n[CyberSettings] %s\n' "$*"; }
ask() {
    local prompt="$1" answer
    while true; do
        read -r -p "$prompt [y/n]: " answer
        case "$answer" in y|Y|yes|YES) return 0;; n|N|no|NO) return 1;; esac
        printf 'Please answer y or n.\n'
    done
}

printf '[CyberSettings] Uninstaller\n'
printf 'This removes only CyberSettings command links and its installation manifest.\n'
printf 'It does not remove CyberShot, CyberClip, CyberNotes, SSH/GPG keys, or user data.\n\n'

scope=""
while [[ "$scope" != "user" && "$scope" != "system" ]]; do
    printf 'Uninstall scope:\n  1) Current user\n  2) System-wide (all users; requires sudo)\n'
    read -r -p 'Choose [1/2]: ' choice
    case "$choice" in
        1) scope="user";;
        2) scope="system";;
        *) printf 'Choose 1 or 2.\n';;
    esac
done

if [[ "$scope" == "user" ]]; then
    DEST="$HOME/.local/bin"
    MANIFEST="$HOME/$MANIFEST_NAME"
else
    [[ "$EUID" -eq 0 ]] || {
        printf 'For system-wide removal, rerun with: sudo %s\n' "$0" >&2
        exit 1
    }
    DEST="/usr/local/bin"
    MANIFEST="/var/lib/cybersettings/$MANIFEST_NAME"
fi

declare -A managed=()
managed_binary=""
if [[ -f "$MANIFEST" ]]; then
    while IFS='=' read -r key value; do
        [[ "$key" == "command" ]] && managed["$value"]=1
        [[ "$key" == "binary" ]] && managed_binary="$value"
    done < "$MANIFEST"
else
    say "No install manifest found. Checking the selected install directory for CyberSettings names."
fi

found=()
for name in "$APP" "$ALIAS"; do
    path="$DEST/$name"
    if [[ -e "$path" || -L "$path" ]]; then
        found+=("$name")
        say "Found '$name':"
        ls -ld -- "$path" || true
    fi
done

if ((${#found[@]} == 0)); then
    say "No CyberSettings command links found in $DEST."
else
    printf '\nWhich command names should be removed?\n'
    printf '  1) cybersettings only\n  2) cse only\n  3) both found commands\n  4) leave all commands installed\n'
    while true; do
        read -r -p 'Choose [1/2/3/4]: ' choice
        [[ "$choice" =~ ^[1-4]$ ]] && break
        printf 'Choose 1, 2, 3, or 4.\n'
    done

    remove=()
    case "$choice" in
        1) [[ -e "$DEST/$APP" || -L "$DEST/$APP" ]] && remove+=("$APP");;
        2) [[ -e "$DEST/$ALIAS" || -L "$DEST/$ALIAS" ]] && remove+=("$ALIAS");;
        3) remove=("${found[@]}");;
        4) remove=();;
    esac

    for name in "${remove[@]}"; do
        path="$DEST/$name"
        # Only remove symlinks; never delete an unrelated regular executable.
        if [[ -L "$path" ]]; then
            target="$(readlink -f -- "$path" 2>/dev/null || true)"
            if [[ "$target" == *"/cybersettings" || "${managed[$name]:-}" == 1 ]]; then
                if ask "Remove command link '$path'?"; then
                    rm -- "$path"
                    say "Removed $path"
                fi
            else
                say "Leaving $path: it does not look like a managed CyberSettings link."
            fi
        else
            say "Leaving $path: it is not a symbolic link."
        fi
    done
fi

remaining_links=0
for name in "$APP" "$ALIAS"; do
    path="$DEST/$name"
    if [[ -L "$path" && "$(readlink -f -- "$path" 2>/dev/null || true)" == "$managed_binary" ]]; then
        remaining_links=1
    fi
done
if [[ -n "$managed_binary" && -f "$managed_binary" && "$remaining_links" -eq 0 ]]; then
    if ask "Remove the CyberSettings binary installed at '$managed_binary'?"; then
        rm -f -- "$managed_binary"
        rmdir "$(dirname "$managed_binary")" 2>/dev/null || true
        rmdir "$(dirname "$(dirname "$managed_binary")")" 2>/dev/null || true
        say "Removed installed binary."
    fi
fi

if [[ -f "$MANIFEST" ]] && ask "Remove CyberSettings install manifest $MANIFEST?"; then
    rm -f -- "$MANIFEST"
fi

if [[ "$scope" == "system" && -d /var/lib/cybersettings ]]; then
    rmdir /var/lib/cybersettings 2>/dev/null || true
fi

say "Uninstall operation finished."
printf 'No other projects or key material were targeted.\n'
