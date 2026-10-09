# Building Zealot

## Backend (Rust)

```bash
# Development
cargo run -p zealot-server

# Release
cargo build --release -p zealot-server
```

The server listens on port **8456** by default.

---

## Linux desktop AppImage

Build the AppImage in the Ubuntu 22.04 container. This avoids the
incompatible `linuxdeploy` toolchain on rolling-release hosts such as Arch:

```bash
npm run build:desktop:appimage
```

The artifact is written to:

```text
target/release/bundle/appimage/Zealot Desktop_0.1.0_amd64.AppImage
```

Install the current artifact for the logged-in user with:

```bash
npm run install:desktop
```

The installer copies the executable to `~/.local/opt/zealot-desktop`, installs
its icon and XDG desktop entry under `~/.local/share`, and refreshes caches when
the relevant utilities are available. i3/Rofi, XFCE, and KDE all discover that
same user-level launcher. To install a differently named artifact, pass its path
directly:

```bash
./scripts/linux-install.sh /path/to/Zealot.AppImage
```

---

## Web App

```bash
# Development (with HMR, proxies /api to localhost:8456)
npm run dev:web

# Production build → apps/web/dist/
npm run build:web
```

---

## Mobile (Tauri 2.0 — iOS & Android)

### One-time setup

**Install Rust mobile targets:**

```bash
# iOS (requires macOS + Xcode)
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim

# Android (requires Android Studio + NDK)
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

**Install the Tauri CLI** (if not already available via npm):

```bash
# Available in the workspace — no global install needed
npx tauri --version
```

**Apple development team** — your team ID (`E958636EK4`) is already set in
`apps/mobile/src-tauri/tauri.conf.json` under `bundle.iOS.developmentTeam`. It
can also be overridden at build time with `APPLE_DEVELOPMENT_TEAM=<id>`.

**Sign into Xcode (once per machine):** Open Xcode → Settings (⌘,) →
Accounts → + → Apple ID. This lets Xcode create provisioning profiles
automatically (`CODE_SIGN_STYLE = Automatic` in the generated Xcode project).

**After `tauri ios init` regeneration:** The generated files in `gen/apple/`
lose two manual patches. Re-apply them:

```bash
# 1. Re-enable automatic signing
sed -i '' 's/CODE_SIGN_IDENTITY = "iPhone Developer";/CODE_SIGN_IDENTITY = "iPhone Developer";\n\t\t\t\tCODE_SIGN_STYLE = Automatic;/g' \
  apps/mobile/src-tauri/gen/apple/zealot-mobile.xcodeproj/project.pbxproj

# 2. Re-add ATS exception so the HTTP dev server is reachable
/usr/libexec/PlistBuddy -c \
  "Add :NSAppTransportSecurity dict" \
  apps/mobile/src-tauri/gen/apple/zealot-mobile_iOS/Info.plist 2>/dev/null || true
/usr/libexec/PlistBuddy -c \
  "Add :NSAppTransportSecurity:NSAllowsArbitraryLoads bool true" \
  apps/mobile/src-tauri/gen/apple/zealot-mobile_iOS/Info.plist
```

**iOS: initialise the Xcode project** (run once per machine):

```bash
cd apps/mobile
npx tauri ios init
```

**Android: initialise the Android Studio project** (run once per machine):

```bash
cd apps/mobile
npx tauri android init
```

---

### iOS development

```bash
# Launch in iOS Simulator
npm run tauri:ios

# Or directly:
cd apps/mobile && npx tauri ios dev
```

### iOS production build

```bash
npm run tauri:ios:build

# Or:
cd apps/mobile && npx tauri ios build
```

Produces a `.ipa` in `apps/mobile/src-tauri/gen/apple/build/`.

### Installing on a physical iPhone

The `.ipa` is signed for development, so it only installs on devices listed
in the team's provisioning profile.

**New iPhone (once per device):**

1. Plug in the iPhone, unlock it, and tap **Trust** if asked.
2. On the iPhone: Settings → Privacy & Security → **Developer Mode** → on →
   Restart → **Turn On** after reboot. If the toggle is missing, open Xcode →
   Window → Devices and Simulators with the phone connected, then look again.
3. Check the phone is ready (`developerModeStatus: enabled`) and note its
   hardware UDID:

   ```bash
   xcrun devicectl list devices   # Identifier column is the <devicectl-id>
   xcrun devicectl device info details --device <devicectl-id> | grep -E "developerModeStatus|udid"
   ```

4. Register the device with the team. `tauri ios build` does not register new
   devices, so run one Xcode build that does:

   ```bash
   cd apps/mobile/src-tauri/gen/apple
   xcodebuild -project zealot-mobile.xcodeproj -scheme zealot-mobile_iOS \
     -destination 'id=<hardware-udid>' -configuration release \
     -allowProvisioningUpdates -allowProvisioningDeviceRegistration build
   ```

   Only the signing step matters here. If the build fails later (for example
   in the Rust build phase), the device is still registered. Signing errors
   at this step:

   - `No Accounts` — sign Xcode into the Apple ID (see one-time setup).
   - `PLA Update available` — accept the latest Program License Agreement at
     <https://developer.apple.com/account>, then retry.
   - `isn't registered in your developer account` — the
     `-allowProvisioningDeviceRegistration` flag is missing.

5. Rebuild the `.ipa` with `npm run tauri:ios:build` so the profile includes
   the new device.

**Install and launch:**

```bash
IPA=apps/mobile/src-tauri/gen/apple/build/arm64/Zealot.ipa
xcrun devicectl device install app --device <devicectl-id> "$IPA"
xcrun devicectl device process launch --device <devicectl-id> com.zealot.app
```

To confirm a build includes a device before installing:

```bash
unzip -q -o "$IPA" -d /tmp/zealot-ipa
security cms -D -i /tmp/zealot-ipa/Payload/Zealot.app/embedded.mobileprovision | grep <hardware-udid>
```

---

### Android development

```bash
# Launch in Android emulator
npm run tauri:android

# Or directly:
cd apps/mobile && npx tauri android dev
```

### Android production build

```bash
npm run tauri:android:build

# Or:
cd apps/mobile && npx tauri android build
```

Produces an `.apk` / `.aab` in `apps/mobile/src-tauri/gen/android/app/build/outputs/`.

---

## Docker (full stack)

```bash
# Build and start everything
npm run docker

# Build images only
npm run build:docker
```

The compose file starts the backend and serves the web frontend via nginx.

---

## All npm convenience scripts

| Script | What it does |
|---|---|
| `npm run dev:web` | Web dev server (port 5173) |
| `npm run build:web` | Web production build |
| `npm run dev:mobile` | Mobile Vite dev server only (port 5174) |
| `npm run build:mobile` | Mobile frontend production build |
| `npm run tauri:ios` | iOS dev build + simulator |
| `npm run tauri:android` | Android dev build + emulator |
| `npm run tauri:ios:build` | iOS release build |
| `npm run tauri:android:build` | Android release build |
| `npm run typecheck` | TypeScript check across all packages |
