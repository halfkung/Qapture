# Test plan

Run the frontend and Rust checks in the [README](../README.md). Record automated unit and interactive desktop results independently.

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
