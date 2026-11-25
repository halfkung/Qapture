# Architecture

Tauri 2 owns the application lifecycle, dashboard WebView2 surface, tray, and integration with Windows shortcut registration and single-instance signaling. The dashboard is created on demand and released on close. The tray and shortcut service continue running without a WebView. The capture coordinator accepts one session at a time from the dashboard, tray, or either global shortcut.

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Capturing
    Capturing --> Selecting: region
    Capturing --> Exporting: full desktop
    Selecting --> Editing: mouse release
    Editing --> Editing: annotate or copy
    Editing --> Idle: editor closed
    Capturing --> Idle: cancelled or failed
    Selecting --> Idle: cancelled or failed
    Exporting --> Idle: cleanup
```

The capture worker sets its thread to Per-Monitor V2 DPI awareness before invoking GDI or creating the overlay. The union of native monitor bounds defines one signed screen rectangle. The image origin is its top-left corner. Each source monitor is copied to that physical offset inside one top-down 32-bit DIB. Gaps between monitors are black. The overlay covers the same physical union and uses client pixel coordinates directly. DPI affects only the size of its controls. The editor translates pointer coordinates through the displayed-to-native canvas ratio.

| Source | Responsibility |
| --- | --- |
| `native/screen.rs` and `geometry.rs` | Monitor enumeration, physical transforms, screenshot allocation and GDI capture |
| `native/overlay.rs` | Native window and input, frozen bitmap paint, physical selection and dimensions |
| `session.rs` | Threaded capture lifecycle, overlap protection, cancellation and cleanup |
| `native/clipboard.rs` | Bounded retries, exact crop, bottom-up CF_DIB, Windows ownership and privacy marker |
| `editor.rs`, `Editor.tsx`, and `markup.ts` | In-memory crop transfer, native-resolution annotation, and clipboard export |
| `shortcuts.rs` and `settings.rs` | Windows registration, conflict handling, atomic settings replacement |
| `dashboard.rs` and `tray.rs` | Dashboard window lifecycle and system tray commands |
| `diagnostics.rs` | Latest result and opt-in timing-only local log |

The native worker captures before creating the overlay. A second DIB supplies consistent dimmed pixels while the original supplies only the active selection. On mouse release, the worker crops to opaque RGBA and drops both desktop DIBs. Only the editor capability can request the selected crop from memory or submit edited pixels for copying. Closing the editor clears the crop and releases the capture session. The dashboard has a separate capability for capture and shortcut controls. PNG export and history remain backlog items.
