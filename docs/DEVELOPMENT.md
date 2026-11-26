# Development

## Prerequisites

- Rust 1.77+ (`rustup`; `rust-toolchain.toml` pins the stable channel)
- Node.js 20.19+ and [pnpm](https://pnpm.io)
- Tauri system dependencies: <https://tauri.app/start/prerequisites/>
  - Debian/Ubuntu: `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`
  - Windows: Microsoft C++ Build Tools and WebView2
  - macOS: Xcode command line tools

## Bundled scrcpy

`pnpm tauri dev` and `pnpm tauri build` first run `pnpm fetch-scrcpy`, which downloads the scrcpy release pinned in `scripts/scrcpy-version.txt` for your OS into `apps/desktop/src-tauri/resources/scrcpy` (git-ignored, bundled into the installer). It skips the download when that version is already there; `tauri build` uses `--strict` so a failed download stops the build instead of producing an installer without scrcpy.

```bash
cd apps/desktop
pnpm fetch-scrcpy --force                      # re-download
SCRCPY_VERSION=latest pnpm fetch-scrcpy --force   # or a specific tag, e.g. v3.3.4
```

To bump the bundled version edit `scripts/scrcpy-version.txt`. Users get newer versions at runtime anyway, so this only sets the baseline.

## Run

```bash
cd apps/desktop
pnpm install
pnpm tauri dev
```

## Checks

```bash
cd apps/desktop && pnpm check          # svelte-check + TypeScript
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace                 # argument building, adb output parsing
```

## Build installers

```bash
cd apps/desktop
pnpm tauri build
```

Bundles are written to `target/release/bundle/` at the workspace root.

## Icons

`resources/icon.png` is the 1024×1024 source. Regenerate all platform icons with:

```bash
cd apps/desktop
pnpm tauri icon ../../resources/icon.png
```

## Project layout

See [ARCHITECTURE.md](ARCHITECTURE.md).
