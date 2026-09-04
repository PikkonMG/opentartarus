#!/bin/sh
# Packs a release build into OpenTartarus-<version>-x86_64.AppImage.
#
# Run `cargo build --release --workspace` first. Needs appimagetool, either
# on PATH or named in $APPIMAGETOOL. The AppImage holds the window and the
# daemon; the window starts the daemon from the directory it lives in, so
# both come up from one file. The permissions helper is not included: an
# AppImage cannot install a polkit policy, so AppImage users install the udev
# rule by hand as the README describes.
set -eu

root=$(cd "$(dirname "$0")/../.." && pwd)
version=$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -1)
out="$root/target/appimage"
appdir="$out/OpenTartarus.AppDir"
tool=${APPIMAGETOOL:-appimagetool}

rm -rf "$appdir"
mkdir -p "$appdir/usr/bin" "$appdir/usr/share/applications" \
    "$appdir/usr/share/icons/hicolor/scalable/apps"

cp "$root/target/release/opentartarus-daemon" "$root/target/release/opentartarus-ui" "$appdir/usr/bin/"
cp "$root/packaging/desktop/opentartarus.desktop" "$appdir/opentartarus.desktop"
cp "$root/packaging/desktop/opentartarus.desktop" "$appdir/usr/share/applications/"
cp "$root/packaging/icons/opentartarus.svg" "$appdir/opentartarus.svg"
cp "$root/packaging/icons/opentartarus.svg" "$appdir/usr/share/icons/hicolor/scalable/apps/"

cat > "$appdir/AppRun" <<'EOF'
#!/bin/sh
here=$(dirname "$(readlink -f "$0")")
exec "$here/usr/bin/opentartarus-ui" "$@"
EOF
chmod 755 "$appdir/AppRun"

ARCH=x86_64 "$tool" "$appdir" "$out/OpenTartarus-$version-x86_64.AppImage"
echo "built $out/OpenTartarus-$version-x86_64.AppImage"
