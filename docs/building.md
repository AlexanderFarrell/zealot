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
