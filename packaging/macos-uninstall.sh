#!/bin/sh
set -eu

if [ "$(uname -s)" != Darwin ]; then
    echo "This uninstaller is for macOS." >&2
    exit 1
fi

if [ "$(id -u)" -ne 0 ]; then
    echo "Run: sudo /usr/local/bin/grid9-uninstall" >&2
    exit 1
fi

app=/Applications/Grid9.app
uninstall_app='/Applications/Uninstall Grid9.app'
cli=/usr/local/bin/grid9
account=${SUDO_USER:-$(id -un)}

# The graphical uninstaller supplies the invoking user before elevation.
if [ "$#" -gt 0 ]; then
    if [ "$#" -ne 2 ] || [ "$1" != --user ]; then
        echo "Usage: grid9-uninstall [--user short-name]" >&2
        exit 1
    fi
    account=$2
fi
case "$account" in
    ''|-*|*[!a-zA-Z0-9._-]*)
        echo "Invalid account name." >&2
        exit 1
        ;;
esac
account_uid=$(id -u "$account")
if [ "$account_uid" -eq 0 ]; then
    echo "Run this uninstaller for a non-root user." >&2
    exit 1
fi

# Check dscl separately: a pipeline would hide its failure behind sed's status.
home_record=$(/usr/bin/dscl . -read "/Users/$account" NFSHomeDirectory)
user_home=$(printf '%s\n' "$home_record" | sed 's/^NFSHomeDirectory: //')
case "$user_home" in
    /*) ;;
    *)
        echo "Could not resolve the home directory for $account." >&2
        exit 1
        ;;
esac
if [ "$user_home" = / ]; then
    echo "Refusing to use / as the user's home directory." >&2
    exit 1
fi
data_dir="$user_home/Library/Application Support/Grid9"

# Validate all installed paths before removing anything.
if [ -e "$app" ] || [ -L "$app" ]; then
    identifier=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$app/Contents/Info.plist")
    if [ "$identifier" != com.treymouledoux.grid9 ]; then
        echo "Refusing to remove an app with a different bundle identifier." >&2
        exit 1
    fi
fi
if [ -e "$uninstall_app" ] || [ -L "$uninstall_app" ]; then
    identifier=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$uninstall_app/Contents/Info.plist")
    if [ "$identifier" != com.treymouledoux.grid9.uninstaller ]; then
        echo "Refusing to remove an uninstaller with a different bundle identifier." >&2
        exit 1
    fi
fi
if [ -e "$cli" ] || [ -L "$cli" ]; then
    if [ ! -L "$cli" ] || [ "$(readlink "$cli")" != "$app/Contents/MacOS/grid9" ]; then
        echo "Refusing to remove a terminal command not linked to Grid9.app." >&2
        exit 1
    fi
fi

# Drop privileges before traversing user-controlled directories, including
# symlinked parents. A linked Grid9 folder is preserved, as on Linux.
cleanup='
set -eu
data=$1
if [ -L "$data" ]; then
    echo "Grid9: skipping symlinked data directory." >&2
    exit 0
fi
rm -rf -- "$data"
'
/usr/bin/sudo -u "$account" -- /bin/sh -c "$cleanup" grid9-cleanup "$data_dir"

if [ -L "$cli" ]; then
    rm "$cli"
fi
rm -rf "$app"
rm -rf "$uninstall_app"
if /usr/sbin/pkgutil --pkg-info com.treymouledoux.grid9.installer >/dev/null 2>&1; then
    /usr/sbin/pkgutil --forget com.treymouledoux.grid9.installer
fi
rm -f /usr/local/bin/grid9-uninstall
echo "Grid9 app and terminal command removed; user-data cleanup completed for $account."
