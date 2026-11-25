# Qapture

**Capture. Create. Share.**

Qapture is a local-first Windows screenshot and annotation project. Phase 0 validates the path from a global shortcut to a native-resolution image on the clipboard.

## Use the prototype

1. Launch `qapture.exe`. You can close the dashboard and continue capturing from the tray.
2. Press **Ctrl+Shift+F9** and drag a region. The overlay shows its physical dimensions. Press **Enter**, use **Ctrl+C**, or select **Copy** to export. Press **Esc** to cancel.
3. Press **Ctrl+Shift+F10** to copy the full desktop across all attached monitors.
4. To change a shortcut, open Qapture from the tray. Choose **Edit** beside an action under **Keyboard shortcuts**, press or type a new combination, and select **Save**. Press **Esc** or select **Cancel** to discard the edit. Typing also lets you check a shortcut already reserved by Windows or another application.

Paste into an image-capable application such as Paint. Qapture publishes the image to the Windows clipboard and requests exclusion from Windows clipboard history and cloud sync.

## Develop on Windows

Use Node.js 22 LTS (22.13+ recommended), Rust stable for `x86_64-pc-windows-msvc`, Visual Studio C++ Build Tools with a Windows SDK, and WebView2 Runtime. The Phase 0 workstation used the workspace-local environment recorded in [project status](docs/PROJECT-STATUS.md). [Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/)

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

Launch `src-tauri/target/release/qapture.exe` after a conventional release build. The NSIS bundle is under `src-tauri/target/release/bundle/nsis`. Pass `--tray` to the executable to start without creating the dashboard.

In the kickoff workspace, the built executable and installer are under `.local/release-target/release`. See the [project status](docs/PROJECT-STATUS.md) for the paths and benchmarks.

See the [project status](docs/PROJECT-STATUS.md) for verified results and the [test plan](docs/TEST-PLAN.md) for interactive checks.
