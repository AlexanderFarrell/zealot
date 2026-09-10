#!/usr/bin/env bash
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
artifact=${1:-"$repo_root/target/release/bundle/appimage/Zealot Desktop_0.1.0_amd64.AppImage"}

if [[ ! -f "$artifact" ]]; then
  printf 'AppImage not found: %s\n' "$artifact" >&2
  printf 'Build it first with: ./scripts/build-desktop-appimage\n' >&2
  exit 1
fi

data_home=${XDG_DATA_HOME:-"$HOME/.local/share"}
app_home=${XDG_OPT_HOME:-"$HOME/.local/opt"}
app_dir="$app_home/zealot-desktop"
app_image="$app_dir/Zealot-Desktop.AppImage"
icon_dir="$data_home/icons/hicolor/512x512/apps"
desktop_dir="$data_home/applications"
icon_name=com.zealot.desktop
desktop_file="$desktop_dir/$icon_name.desktop"

mkdir -p "$app_dir" "$icon_dir" "$desktop_dir"
install -m 0755 "$artifact" "$app_image"
install -m 0644 "$repo_root/apps/desktop/src-tauri/icons/icon.png" "$icon_dir/$icon_name.png"

cat > "$desktop_file" <<EOF
[Desktop Entry]
Type=Application
Version=1.0
Name=Zealot Desktop
Comment=Local-first work management
Exec=$app_image
Icon=$icon_name
Terminal=false
Categories=Office;
StartupWMClass=Zealot
EOF

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$desktop_dir" >/dev/null 2>&1 || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache --force "$data_home/icons/hicolor" >/dev/null 2>&1 || true
fi

printf 'Installed Zealot Desktop:\n  AppImage: %s\n  Launcher: %s\n' "$app_image" "$desktop_file"
