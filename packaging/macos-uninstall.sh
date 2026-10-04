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
cli=/usr/local/bin/grid9
account=${SUDO_USER:-$(id -un)}
user_home=$(/usr/bin/dscl . -read "/Users/$account" NFSHomeDirectory | sed 's/^NFSHomeDirectory: //')
case "$user_home" in
    /*) ;;
    *) echo "Could not resolve the home directory for $account." >&2; exit 1 ;;
esac
if [ "$user_home" = / ]; then
    echo "Refusing to use / as the user's home directory." >&2
    exit 1
fi
data_dir="$user_home/Library/Application Support/Grid9"
if [ -e "$app" ] || [ -L "$app" ]; then
    identifier=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$app/Contents/Info.plist")
    if [ "$identifier" != com.treymouledoux.grid9 ]; then
        echo "Refusing to remove an app with a different bundle identifier." >&2
        exit 1
    fi
fi
if [ -e "$cli" ] || [ -L "$cli" ]; then
    if [ ! -L "$cli" ] || [ "$(readlink "$cli")" != "$app/Contents/MacOS/grid9" ]; then
        echo "Refusing to remove a terminal command not linked to Grid9.app." >&2
        exit 1
    fi
    rm "$cli"
fi
rm -rf "$app"
rm -rf "$data_dir"
if /usr/sbin/pkgutil --pkg-info com.treymouledoux.grid9.installer >/dev/null 2>&1; then
    /usr/sbin/pkgutil --forget com.treymouledoux.grid9.installer
fi
rm -f /usr/local/bin/grid9-uninstall
echo "Grid9 app, terminal command, and user data removed for $account."
