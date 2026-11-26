# Third-party software

## Bundled and downloaded at runtime

The installer includes, and the auto-updater downloads, the **unmodified official scrcpy release archive** from
<https://github.com/Genymobile/scrcpy/releases>. That archive contains:

| Component | License |
| --- | --- |
| scrcpy and scrcpy-server | Apache License 2.0 (Genymobile) |
| adb (Android platform-tools) | Apache License 2.0, distributed under the [Android SDK terms](https://developer.android.com/studio/terms) |
| FFmpeg libraries, SDL, libusb and others | their own licenses (mostly LGPL / zlib / BSD), see the scrcpy documentation |

`scripts/fetch-scrcpy.mjs` stores the scrcpy `LICENSE` next to the bundled files (`LICENSE.scrcpy`).
This is a summary for convenience, not legal advice: check the licenses of the exact release you redistribute.

## Rust and JavaScript dependencies

See `Cargo.lock` and `pnpm-lock.yaml` for the exact versions. Main ones: Tauri (MIT/Apache-2.0), Svelte (MIT),
reqwest, tokio, serde, sha2, zip, tar, flate2 (MIT/Apache-2.0).
