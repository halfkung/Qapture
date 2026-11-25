# Product requirements

Qapture gives Windows 10/11 users a fast, offline workflow for capturing and annotating screenshots. Every user gets the same features. Core workflows require no account, server, or network.

The primary flow is global shortcut → frozen desktop → region → editable annotations → copy or save. The shortcut and tray work with the dashboard closed. Capture precedes overlay display so visible menus and tooltips can survive in the frozen frame. Desktop coordinates and output sizes use physical pixels.

## Release scope

| Milestone | Deliverables |
| --- | --- |
| Phase 0 | Stack feasibility, native capture and selection, tray, two configurable hotkeys with conflicts reported, native clipboard, native-scale crop, cancellation, session recovery, benchmarks |
| v0.1 Foundation | Region and full screen, monitor and DPI support, resizable selection with dimensions, floating annotation tools, undo/redo, copy, PNG, settings, Windows installer |
| v0.2 Productivity | Local history, window capture, step numbers, blur and mosaic, opaque redaction, import, pin, delay, auto save, presets |
| v0.3 Advanced editing | Side-by-side and expected/actual layouts, stitching, multiple-image canvas, scrolling capture, Thai/English OCR, magnifier, spotlight, JPG/WebP |
| v0.4 Smart tools | Screen recording, GIF, templates, automation actions, optional AI assistance, external integrations |

The foundation editor supports rectangles, ellipses, arrows, lines, freehand, highlighter, and text. Annotations stay selectable, movable, resizable, and individually editable. Text supports Thai and English, multiline input, and font sizing. Shape properties include color and stroke width.

## Initial acceptance targets

| Metric | Target |
| --- | --- |
| Warm hotkey to overlay | p95 ≤ 200 ms |
| 1080p clipboard export | ≤ 300 ms |
| Cold startup | ≤ 2 s |
| Idle CPU | ≤ 1% |
| Idle memory including relevant processes | ≤ 100 MB stretch |
| Controlled capture reliability | ≥ 99.9% |
| Display coverage | FHD, 2K, 4K and 100%–200% scale |
