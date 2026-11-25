# Qapture

**Capture. Create. Share.**

Qapture is a local-first Windows screenshot and annotation project. Phase 0 validates the path from a global shortcut to a native-resolution image on the clipboard.

## Develop on Windows

Install Node.js 22 LTS, Rust stable for `x86_64-pc-windows-msvc`, Visual Studio C++ Build Tools with a Windows SDK, and WebView2 Runtime. [Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/)

```powershell
npm ci
npm run tauri -- dev
```

```powershell
npm run lint
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri -- build
```

See the [project status](docs/PROJECT-STATUS.md) for verified results and the [test plan](docs/TEST-PLAN.md) for interactive checks.
