# Qapture

**Capture. Create. Share.**

Qapture is a local-first Windows capture and markup app. Its quick editor lets you mark up a selected region and copy it at native resolution.

## Install or update

Run the x64 NSIS installer for the new version. To update an existing installation, choose **Quit Qapture** in the tray before running the installer, and select the option to keep the existing installation if prompted. The installer and a standalone executable are listed in [project status](docs/PROJECT-STATUS.md).

## Use Qapture

1. Launch `qapture.exe`. You can close the dashboard and continue capturing from the tray.
2. Press **Ctrl+Shift+F9** and drag a region. The selection stays bright while the desktop around it is dimmed. Release the mouse to open the quick editor, where you can use a pen, rectangle, or arrow and then select **Copy image** or press **Ctrl+C**. Press **Esc** in the overlay to cancel or in the editor to close it.
3. Press **Ctrl+Shift+F10** to copy the full desktop across all attached monitors.
4. To change a shortcut, open Qapture from the tray. Choose **Edit** beside an action under **Keyboard shortcuts**, press the new combination on your keyboard, and select **Save**. Press **Esc** or select **Cancel** to discard the edit.
5. If Windows reports **Shortcut already in use**, choose **Use available shortcut**. Qapture retries the displayed combination, then tries alternatives and displays the one it registered. To keep the exact combination, first release it in the other app. The dashboard capture buttons also work while a shortcut is unavailable.

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

See the [project status](docs/PROJECT-STATUS.md) for verified results and the [test plan](docs/TEST-PLAN.md) for interactive checks.
