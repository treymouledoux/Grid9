#!/bin/sh
# dpkg calls postrm for upgrades as well as removal; never clean during upgrades.
set -eu
case "${1:-}" in remove|purge) ;; *) exit 0 ;; esac
skip() { echo "Grid9: $*; user data left unchanged." >&2; exit 0; }
[ -n "${SUDO_USER:-}" ] && [ -n "${SUDO_UID:-}" ] || skip "no verified sudo user"
case "$SUDO_UID" in ''|*[!0-9]*|0) skip "invalid sudo user" ;; esac
actual_uid=$(id -u -- "$SUDO_USER" 2>/dev/null) || skip "unknown sudo user"
[ "$actual_uid" = "$SUDO_UID" ] || skip "sudo identity does not match"
# Explicit portable data directories are never removed by package management.
[ -z "${GRID9_DATA_DIR:-}" ] || skip "custom GRID9_DATA_DIR"
entry=$(getent passwd "$SUDO_USER") || skip "cannot resolve user home"
user_home=$(printf '%s\n' "$entry" | cut -d: -f6)
case "$user_home" in /*) ;; *) skip "invalid user home" ;; esac
[ "$user_home" != / ] || skip "invalid user home"
data_base=${XDG_DATA_HOME:-"$user_home/.local/share"}
case "$data_base" in /*) ;; *) skip "XDG_DATA_HOME is not absolute" ;; esac
# Never recursively traverse a user-controlled path with root privileges.
cleanup='
set -eu
data=$1/Grid9
[ ! -L "$data" ] || { echo "Grid9: skipping symlinked data directory." >&2; exit 0; }
for component in documentation logs preprocessor_cache; do
    rm -rf -- "$data/$component"
done
rm -f -- "$data/.components-revision"
echo "Grid9: removed generated documentation, logs, and cache; examples and personal files preserved."
'
if [ "$(id -u)" = 0 ]; then
    # postrm must also work after dependencies have been removed on purge.
    command -v runuser >/dev/null 2>&1 || skip "runuser unavailable"
    if ! runuser -u "$SUDO_USER" -- sh -c "$cleanup" grid9-cleanup "$data_base"; then
        echo "Grid9: some user data could not be cleaned; package removal will continue." >&2
    fi
elif [ "$(id -u)" = "$actual_uid" ]; then
    sh -c "$cleanup" grid9-cleanup "$data_base" || true
else
    skip "cleanup must run as the original user or root"
fi
exit 0
