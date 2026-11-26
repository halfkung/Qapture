# Technical decisions

## Tauri shell and native capture surface

Keep Tauri 2 with React, TypeScript, Vite and Rust. A native monitor-aware overlay displays the captured frame directly on the capture worker. This avoids encoding the desktop and transferring it through WebView2 before selection. Close the dashboard's WebView when the user dismisses it to reduce tray memory. Create another WebView when the user reopens the dashboard. The preliminary hidden-WebView implementation measured 375 MiB of combined working set on the test workstation.

## Capture backend options

| Backend | Strengths | Costs and limits |
| --- | --- | --- |
| GDI BitBlt with CAPTUREBLT | Direct CPU bitmap from the composed desktop and a practical first-frame path for small screenshots. Layered windows can be included. | CPU copies, limited color handling, and variable behavior with protected surfaces, GPU overlays, or remote desktops. |
| Windows Graphics Capture | WinRT capture of a monitor or window. Monitor interop entry point begins with Windows 10 version 1903. | Per-target frame sessions, D3D11 readback, startup timing, and capture border policy need measurement. |
| DXGI Desktop Duplication | GPU frames per output and dirty/move metadata, a good fit for continuous capture. | Multiple adapters and outputs require coordination. First-frame waits, rotation, access loss, and staging/readback add work. |

Select GDI for Phase 0 so a tray-resident app can capture on demand with no GPU frame subscription. The WGC and DXGI evaluations were documentation reviews. Start a comparative prototype with a representative fixture if capture accuracy, HDR, or reliability fails on target hardware. [GDI BitBlt](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-bitblt) · [Windows Graphics Capture](https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture) · [Win32 capture interop](https://learn.microsoft.com/en-us/windows/win32/api/windows.graphics.capture.interop/nn-windows-graphics-capture-interop-igraphicscaptureiteminterop) · [Desktop Duplication](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api)

## Clipboard formats

Use the Windows device-independent bitmap format to provide native-size pixels and common paste compatibility. Mark the result for exclusion from built-in Windows clipboard history and cloud sync. [Windows clipboard formats](https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-formats)

## Dashboard shortcut editing

Keep shortcut editing within the dashboard with a focusable key recorder. While the focused dashboard is recording, Qapture routes its own registered hotkeys into the recorder. Native validation and persistence remain in the shortcut service.

Open a 920 × 740 logical-pixel dashboard constrained by the monitor work area. Use a shorter banner in short windows and collapse the banner while editing at the minimum height. Reserve space for conflict actions. Keep the content pane accessible when magnification or exceptional content exceeds the available space.

Preserve saved shortcuts on startup conflicts. User-initiated recovery retries the saved combination first, skips the other action's binding and tries alternatives only on an actual registration conflict. The Windows plugin exposes this native error as text. Keep that classification in the Rust service. Commit a replacement only after registration and settings persistence succeed.

## Selection and quick editor

Paint the backdrop from a pre-dimmed GDI bitmap and paint the selection from the original snapshot. This gives the same dimming across monitors and keeps the selected pixels unchanged. Crop on pointer release and pass only the crop through Tauri's binary IPC to an editor-only capability. Use a second Canvas 2D layer for vector marks and keep the original pixels separately. Reuse the native Windows clipboard path for the composited image.

## Editable documents and export

Represent marks with IDs, source-image coordinates, and properties. Store immutable document changes for undo and redo. The visible canvas composites onto a separate bitmap. Copy, file export, and pin render a flattened result at native resolution. The UI owns selection and crop outlines, so those overlays stay out of the export. Solid fills write opaque pixels across integer pixel bounds.

Window capture selects a visible rectangle in the same frozen desktop snapshot using the Windows z-order. The capture session keeps a lease during the delay, selection and editing. The standard region path still opens the editor on mouse release. The adjustment option adds move, resize and confirmation to the native overlay.

Export PNG, JPG or WebP using WebView2 encoding and pass final bytes to scoped Rust commands. Use a dedicated Windows COM thread for the native save picker. Configured-folder exports write to unique temporary files and then rename them. Retained Qapture projects are opt-in local files with a JSON annotation header and the original pixel packet. The History UI exposes clearing and warns that editable files contain original pixels.

## Alternative stack

If measured WebView2 overhead or window lifecycle behavior blocks agreed targets, compare C# with WPF and Win32 interop using the same fixture. WPF could replace the dashboard and editor host and would still need native capture, DPI, and clipboard integration. Switching the primary stack requires Product Owner approval.
