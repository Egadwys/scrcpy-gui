# Contributing

1. Fork and create a branch.
2. Follow [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) to run the app locally.
3. Before opening a pull request make sure these pass:

```bash
cd apps/desktop
pnpm check
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

Keep changes focused. New scrcpy options belong in `LaunchOptions` (Rust, `settings.rs`), `build_args` (`scrcpy.rs`, with a test), the `LaunchOptions` type (`types.ts`) and `OptionsPanel.svelte`.
