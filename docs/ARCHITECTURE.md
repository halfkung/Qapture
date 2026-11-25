# Architecture

Tauri 2 owns the application lifecycle, dashboard WebView2 surface, tray, and integration with Windows shortcut registration and single-instance signaling. The dashboard is created on demand and released on close. The tray and shortcut service continue running without a WebView. The capture coordinator accepts one session at a time from the dashboard, tray, or either global shortcut.

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Capturing
    Capturing --> Selecting: region
    Capturing --> Exporting: full desktop
    Selecting --> Exporting: Phase 0 copy
    Selecting --> Annotating: Phase 1
    Annotating --> Exporting
    Capturing --> Idle: cancelled or failed
    Selecting --> Idle: cancelled or failed
    Exporting --> Idle: cleanup
```

The capture worker sets its thread to Per-Monitor V2 DPI awareness before invoking GDI or creating the overlay. The union of native monitor bounds defines one signed screen rectangle. The image origin is its top-left corner. Each source monitor is copied to that physical offset inside one top-down 32-bit DIB. Gaps between monitors are black. The overlay covers the same physical union and uses client pixel coordinates directly. DPI affects only the size of its controls. Future editor code should convert via its actual viewport-to-backing-pixel ratio.

| Source | Responsibility |
| --- | --- |
| `native/screen.rs` and `geometry.rs` | Monitor enumeration, physical transforms, screenshot allocation and GDI capture |
| `native/overlay.rs` | Native window and input, frozen bitmap paint, physical selection and dimensions |
| `session.rs` | Threaded capture lifecycle, overlap protection, cancellation and cleanup |
| `native/clipboard.rs` | Bounded retries, exact crop, bottom-up CF_DIB, Windows ownership and privacy marker |
| `shortcuts.rs` and `settings.rs` | Windows registration, conflict handling, atomic settings replacement |
| `dashboard.rs` and `tray.rs` | Dashboard window lifecycle and system tray commands |
| `diagnostics.rs` | Latest result and opt-in timing-only local log |

The native worker captures before creating the overlay. Copy reads the selected rows from the original capture buffer. Screenshot pixels stay outside the WebView and are dropped with the worker. The dashboard capability exposes five application commands and event subscription to the local `main` window. PNG export, annotation, and optional history storage are separate backlog items.
