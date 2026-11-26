# Changelog

## 0.2.0 — Windows preview

- Added visible-window capture, delayed capture, optional region adjustment, and a pixel magnifier.
- Added selectable annotations, Thai and English multiline text, layer controls, crop, zoom, editor shortcuts, and undo/redo for document changes.
- Added local PNG, JPG and WebP export, optional Auto Save, editable project retention and search, image import, named annotation presets, and a pinned preview.
- Added the Window and History shortcuts and an opt-in Start with Windows setting. Existing shortcut settings carry forward.

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
