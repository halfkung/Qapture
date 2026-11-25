# Test plan

Run the frontend and Rust checks in the [README](../README.md). Record automated unit and interactive desktop results independently.

Run the automated Windows smoke check from an unlocked test desktop with Qapture closed and the default shortcuts available. The fixture briefly takes focus and moves the pointer. It reads only known fixture pixels and logs only outcomes and timings.

```powershell
powershell.exe -NoProfile -STA -File scripts/Test-Desktop.ps1 -Executable src-tauri/target/release/qapture.exe
```

Add `-TestClipboard` on a test desktop with an empty clipboard to check successful region and full-desktop copies. The default run exercises clipboard contention without replacing the current clipboard item. To check interoperability manually, paste a controlled capture into Paint and compare its dimensions with the overlay.

| Desktop scenario | Expected result |
| --- | --- |
| App open and dropdown expanded | Frozen image contains the visible surface present at trigger time |
| Drag both directions | Overlay displays native pixel dimensions and clipboard contains the exact crop |
| Repeat shortcut during selection | One selection window remains active |
| Escape, then retry | Existing frame is released and the next trigger succeeds |
| Clipboard copy | Paste reproduces image dimensions and fixture colors |
| Full desktop | Output covers both monitor rectangles and pads topology gaps |
| Different monitor scales | Region start and end align to the pointer on both monitors and across their seam |
| Another process owns a key | Conflict is shown, the previous binding remains active |
| Launch twice | The first instance retains tray and hotkey ownership |
| Tray while dashboard is closed | Capture, dashboard restore, and quit are available |

Record OS build, physical topology, scaling, and build type. Use a controlled image with distinct sample colors for accuracy. Report capture stage timings with sample counts and nearest-rank p95. For idle memory, report Qapture and associated WebView2 process working sets along with dashboard visibility. Treat process launch to webview readiness separately from first overlay paint.
