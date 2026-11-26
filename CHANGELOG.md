# Changelog

## 0.1.2 — 26 November 2026

- Open the dashboard at 920 × 740 logical pixels, bounded by the monitor work area. Compact layouts fit both shortcut conflicts and the expanded recorder.
- Show a concise registration error and let the user recover an unavailable shortcut in one click. Retry the saved chord first and persist an alternative only after Windows accepts it.

## 0.1.1 — 26 November 2026

- Added a capture illustration and shortcut settings link to the dashboard.
- Replaced text shortcut editing with direct keyboard recording and keycaps, preserving validation and focus restoration.
- Increased dimming outside the active selection and opened the quick editor on mouse release.
- Added native-resolution pen, rectangle, arrow, undo, redo, and clipboard export in the quick editor.
- Made the dashboard fit its default and minimum window sizes without a scrollbar. Expanded settings can still scroll.

## 0.1.0 — Phase 0 prototype

- Established the Tauri 2, React, TypeScript, Vite, Tailwind CSS, and Rust project.
- Added tray and single-instance lifecycle, persistent configurable shortcuts, and Windows registration feedback.
- Added native screenshot-backed multi-monitor region selection, full desktop capture, cancellation, and clipboard export at physical resolution.
- Added desktop smoke checks, opt-in timing diagnostics, and an NSIS packaging configuration.
