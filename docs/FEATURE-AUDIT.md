# Qapture feature audit

Baseline is 0.1.2 at `81a0e6b`. The worktree was clean. The packaged dashboard was running on Windows. Verification below includes prior recorded desktop checks where noted. Source entry points are `App.tsx`, `Editor.tsx`, `session.rs`, `native/overlay.rs`, `shortcuts.rs`, and the tray and settings modules.

| Baseline feature | Classification | Evidence |
| --- | --- | --- |
| Region and full desktop | IMPLEMENTED AND VERIFIED | Native GDI snapshot, region-to-editor, full-desktop clipboard format and contention path. External paste remains pending. |
| Multiple monitors, mixed DPI and negative origins | IMPLEMENTED AND VERIFIED | Prior 100%/125% Windows fixture across the display seam. Windows 10, 4K and 200% qualification remain pending. |
| Frozen overlay, menus, dimensions, cancel, repeat and session recovery | IMPLEMENTED AND VERIFIED | Captures before overlay creation. Prior fixture checked popup pixels, dimming, exact crop and repeated sessions. |
| Window capture, delay, movable/resizable selection and magnifier | MISSING | Region selection commits on pointer release. |
| Pen, rectangle, arrow and creation undo/redo | IMPLEMENTED AND VERIFIED | Quick editor and controlled Windows interaction. |
| Select, ellipse, line, highlighter, text, numbered steps, blur, mosaic, redaction, crop | MISSING | Only the three quick editor tools exist. |
| Object movement, resize, delete, duplicate, reordering, opacity and property editing | MISSING | Marks are append-only and have no identity. Color and width set future marks. |
| Non-destructive image model and undo | PARTIALLY IMPLEMENTED | Original pixels are separate. History only handles newly added marks. |
| Thai/English text, multiline, styles, alignment and IME | MISSING | Editor has no text input. |
| Editor controls, light/dark and dimensions | PARTIALLY IMPLEMENTED | The toolbar, tooltips, active tools, theme and dimensions exist. Fit is automatic. Zoom, contextual properties and remembering tool preferences are absent. |
| Native clipboard | PARTIALLY IMPLEMENTED | Native DIB and native-size compositing exist. The current clipboard was preserved during testing, so independent paste and 1080p publication latency remain pending. |
| PNG, JPG, WebP, naming, path and auto save | MISSING | There is no save command. |
| History, local retention, search, delete, reopening projects, image import, pinning and presets | MISSING | No history or storage UI. |
| Tray, global keys, editable keys, conflict recovery, dashboard, persisted settings, single instance, background operation and quit | IMPLEMENTED AND VERIFIED | Native tray and settings, registered bindings, close to tray and restart covered by prior checks. Defaults differ from the new brief. |
| Start with Windows | MISSING | No login registration. |
| Windows installer | IMPLEMENTED BUT UNVERIFIED | Unsigned x64 NSIS artifact built. Clean install, upgrade and removal on the release matrix pending. |
| Bedrock UI/backend integration | IMPLEMENTED AND VERIFIED | Existing visible features call live IPC commands scoped by Tauri window capability. No placeholder editor tools found. |
| Sleep/wake, current performance and reliability qualification | IMPLEMENTED BUT UNVERIFIED | Historical timings in project status. Current build needs remeasurement. |

## Advanced roadmap

Baseline status for every item is **MISSING**.

| Feature | Feasibility, dependencies and remaining work |
| --- | --- |
| Expected vs Actual and side-by-side | Reuse local image decoding and native-pixel rendering. Define independent layout and captions before exposing export. |
| Stitching and multiple-image canvas | Add a document layout and bounded memory for multiple buffers. Stitching needs overlap matching and manual correction. |
| Scrolling capture | Needs consented window scrolling, repeatable frame capture and sticky-element handling. Test against representative apps. |
| Thai and English OCR | Evaluate locally installed recognition languages and Thai accuracy. Keep processing offline. |
| Spotlight and advanced magnifier | Can reuse a frozen frame. Validate pointer tracking, color sampling and scaled displays. |
| Screen recording and GIF | Evaluate Windows Graphics Capture, encoder availability, capture indicators and temporary storage. |
| Templates and automation actions | Reuse the document model once stable. Define local actions and explicit file-write settings. |
| AI and integrations | Optional future scope. |

## Implementation after the audit

| Feature | 0.2.0 preview status |
| --- | --- |
| Region, full desktop, tray lifecycle, shortcuts and capture sessions | IMPLEMENTED AND VERIFIED in source checks and controlled Windows workflows. Existing bindings migrate forward. A fresh profile uses Alt+A, Alt+S, Alt+W and Alt+Q. An Alt chord can dismiss a Windows context menu before the frame is sampled. |
| Window capture and delay | IMPLEMENTED AND VERIFIED with a visible-window fixture. GDI captures the composed desktop inside the selected window bounds. |
| Region adjustment and magnifier | IMPLEMENTED AND VERIFIED for native adjustment. Move, lower-right resize and Enter produced the expected crop. M toggled in the fixture. Mixed-DPI magnifier alignment still needs visual qualification. |
| Annotation shapes, transforms, styling, crop and undo/redo | IMPLEMENTED BUT UNVERIFIED across every tool. Source tests exercise history, transforms, Unicode metadata and bounds. The Windows fixture covered drawing, undo/redo, text movement and crop. |
| Thai, English and Unicode text | IMPLEMENTED BUT UNVERIFIED with physical Thai keyboard/IME. The fixture typed multiple lines, visibly rendered Thai marks and reopened editable text. |
| Native clipboard | PARTIALLY IMPLEMENTED for release acceptance. Native full-size and crop-packet paths exist. The cropped contention case passed. Successful inter-app paste and 1080p publication latency remain pending on a disposable clipboard. |
| PNG, JPG, WebP, Auto Save, directory and naming | IMPLEMENTED AND VERIFIED on a controlled image. PNG and JPG dimensions were decoded independently. WebP signature, the flattened opaque redaction and configured output were checked. Native Save As was exercised on the fixture, with latency varying across runs. |
| History, retention, search, deletion, reopen and image import | IMPLEMENTED AND VERIFIED on a controlled image. Users enable editable retention and explicitly keep a project. Auto Save produces a separate flattened file on editor close when enabled. |
| Pin, presets and Start with Windows | IMPLEMENTED BUT UNVERIFIED end to end. Pin creation passed. Login, installer startup-entry removal, and persistent preset interaction need manual verification. |
| Advanced features and public release | BLOCKED by release gates. Clipboard interoperability, sleep/wake, the full Windows and display matrix, release signing and install/upgrade/removal qualification are pending. The advanced feasibility table remains active. |

### Implementation files and outputs

- Native capture and desktop integration are in `session.rs`, `native/overlay.rs`, `native/screen.rs`, `shortcuts.rs`, `settings.rs`, `tray.rs` and `startup.rs`.
- Native export, pinning and editable projects are in `editor.rs`, `export.rs`, `pin.rs` and `storage.rs`. IPC registration and window-specific capabilities are defined in `main.rs`, `build.rs` and `capabilities/`.
- The dashboard and history use `App.tsx`, `History.tsx` and their styles. The editor, preview and document model use `Editor.tsx`, `EditorProperties.tsx`, `Pin.tsx`, `editor-document.ts`, `markup.ts` and `editor.css`.
- Build outputs and measured and pending verification are recorded in [Project status](PROJECT-STATUS.md). The [README](../README.md) covers setup and launch.
