# Technical decisions

## Tauri shell and native capture surface

Keep Tauri 2 with React, TypeScript, Vite and Rust. A native monitor-aware overlay can display the captured frame directly on the capture thread. This avoids encoding an image and transferring it through WebView2 before selection. Review the feasibility decision after measured capture and idle results.

## Capture backend options

| Backend | Strengths | Costs and limits |
| --- | --- | --- |
| GDI BitBlt with CAPTUREBLT | Direct CPU bitmap from the composed desktop and a practical first-frame path for small screenshots. Layered windows can be included. | CPU copies, limited color handling, and variable behavior with protected surfaces, GPU overlays, or remote desktops. |
| Windows Graphics Capture | WinRT capture of a monitor or window. Monitor interop entry point begins with Windows 10 version 1903. | Per-target frame sessions, D3D11 readback, startup timing, and capture border policy need measurement. |
| DXGI Desktop Duplication | GPU frames per output and dirty/move metadata, a good fit for continuous capture. | Multiple adapters and outputs require coordination. First-frame waits, rotation, access loss, and staging/readback add work. |

Select GDI for Phase 0. Revisit with a representative fixture if capture accuracy, HDR, or reliability fails on target hardware. [GDI BitBlt](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-bitblt) · [Windows Graphics Capture](https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture) · [Win32 capture interop](https://learn.microsoft.com/en-us/windows/win32/api/windows.graphics.capture.interop/nn-windows-graphics-capture-interop-igraphicscaptureiteminterop) · [Desktop Duplication](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api)

## Alternative stack

If measured WebView2 overhead or window lifecycle behavior blocks agreed targets, compare C# with WPF and Win32 interop using the same fixture. WPF could replace the dashboard and editor host and would still need native capture, DPI, and clipboard integration. Switching the primary stack requires Product Owner approval.
