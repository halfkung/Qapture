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
