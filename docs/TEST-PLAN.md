# Test plan

Run the frontend and Rust checks in the [README](../README.md). Record automated unit and interactive desktop results independently.

Run the automated Windows smoke check from an unlocked test desktop with Qapture closed and the default shortcuts available. Use `-UseFunctionKeys` after setting Region to Ctrl+Shift+F9 and Full desktop to Ctrl+Shift+F10 to verify frozen menu pixels. The fixture briefly takes focus and moves the pointer. It reads only known fixture pixels and logs only outcomes and timings.

```powershell
powershell.exe -NoProfile -STA -File scripts/Test-Desktop.ps1 -Executable src-tauri/target/release/qapture.exe
```

Add `-TestClipboard` on a test desktop with an empty clipboard to check successful editor and full-desktop copies. The default run exercises full-desktop clipboard contention without replacing the current clipboard item. To check interoperability manually, paste a controlled capture into Paint and compare its dimensions with the overlay.

| Desktop scenario | Expected result |
| --- | --- |
| Open and resize the dashboard | Capture actions and both shortcut rows fit at the default and minimum sizes without scrollbars |
| Edit the lower shortcut with both bindings unavailable | At 920 × 740, 720 × 620 and 540 × 500 both settings rows and the recorder fit with zero scrolling. Cancel returns focus to Edit. |
| Recover startup conflicts | Reserve both saved keys in a second process. Keep one reserved and release the other before recovery. Confirm an alternate for the busy key, reuse of the released key, and both bindings active after restart. |
| App open and dropdown expanded | Frozen image contains the visible surface present at trigger time |
| Drag both directions | The selection stays bright, surrounding pixels dim, and mouse release opens the exact crop in the editor |
| Draw and undo in the editor | Marks appear at the intended location and undo restores the original pixels |
| Record a Qapture hotkey while editing | The pressed combination appears without starting a capture |
| Repeat shortcut during selection | One selection window remains active |
| Escape, then retry | Existing frame is released and the next trigger succeeds |
| Clipboard copy | Paste reproduces image dimensions and fixture colors |
| Full desktop | Output covers both monitor rectangles and pads topology gaps |
| Different monitor scales | Region start and end align to the pointer on both monitors and across their seam |
| Another process owns a key | Conflict is shown, the previous binding remains active |
| Launch twice | The first instance retains tray and hotkey ownership |
| Tray while dashboard is closed | Capture, dashboard restore, and quit are available |

| Editor and productivity scenario | Expected result |
| --- | --- |
| Delay and cancel | A delayed capture acquires the session once, waits, and responds to tray cancellation. |
| Adjust and magnify | Drag a selection, move it, resize from the lower-right corner, toggle M and commit with Enter. Verify pixel dimensions on each monitor. |
| Visible window capture | Hover and click a real window, compare the opened crop to its visible desktop bounds. |
| Text and shortcuts | Type multiline Thai with the Windows keyboard or IME, select and copy text, blur the field, then test editor and global shortcuts. |
| Editable layers | Create, move, resize, change properties, delete, reorder, undo and redo. Reopen enabled History and verify the same objects remain editable. |
| Local export | Cancel the native Save dialog, retry, then independently open PNG, JPG and WebP. Verify output size and inspect a solid-redacted pixel. |
| Storage controls | Verify a fresh profile writes no capture files. Enable local retention or Auto Save, set a folder and file prefix, search/reopen/delete projects, and clear retained files. |
| Pin, presets and login | Verify pin stays above other apps and closes independently, a named preset survives restart, and Start with Windows opens only the tray on next login. |

Record OS build, physical topology, scaling, and build type. Use a controlled image with distinct sample colors for accuracy. Report capture stage timings with sample counts and nearest-rank p95. For idle memory, report Qapture and associated WebView2 process working sets along with dashboard visibility. Treat process launch to webview readiness separately from first overlay paint.
