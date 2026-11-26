# Qapture

**Capture. Create. Share.**

Qapture is a local-first Windows capture and markup app. Its quick editor lets you mark up a selected region and copy it at native resolution.

## Install or update

Run the x64 NSIS installer for the new version. To update an existing installation, choose **Quit Qapture** in the tray before running the installer, and select the option to keep the existing installation if prompted. The installer and a standalone executable are listed in [project status](docs/PROJECT-STATUS.md).

## Use Qapture

1. Run `Qapture.exe`. Closing the dashboard leaves Qapture in the tray. Choose **Quit Qapture** from the tray to exit.
2. Use **Alt+A** for a region, **Alt+S** to copy the full desktop, **Alt+W** to select a visible window, and **Alt+Q** to open History on a new installation. Existing installations keep their recorded keys. Check **Keyboard shortcuts** for the active keys. If a key is busy, use **Edit** or **Use available shortcut**.
3. Drag and release to open a region in the editor. To move or resize before committing, enable **Adjust region before editing** on the dashboard, drag inside the selection or use its lower-right handle, then press **Enter**. Press **M** to toggle the magnifier and **Esc** to cancel. The dashboard **Delay** selector gives you time to open a context menu before capture.
4. In the editor, choose a tool and draw on the canvas. Use **Select** to move or resize an object. For text, click or drag a box, type in **Annotation text**, then use the font and alignment controls. Tooltips show editor shortcuts. Use **Copy image** for the Windows clipboard, **Save** for PNG/JPG/WebP, or the pin button for an always-on-top preview. If you configured an export folder, **Save** writes there and **Save as…** opens the Windows dialog.
5. Open **History** to import images and configure the export folder, file prefix, Auto Save and startup. Enable **Allow saving editable projects** and choose **Keep editable project** in the editor to retain a capture. History allows search, reopening, deletion and clearing local projects.

The clipboard path requests exclusion from Windows clipboard history and cloud sync. Solid redaction covers pixels in flattened exports. Editable History files contain original pixels and can expose redacted content when someone reopens them. Auto Save writes a flattened file when closing the editor if enabled. Full desktop copy uses the clipboard directly.

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
