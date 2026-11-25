# Development plan

## Phase 0

1. Audit the Windows workstation and resolve development prerequisites locally where practical.
2. Initialize Git, documentation, frontend tooling, and the Tauri shell. Build and launch the shell.
3. Implement the native tray and settings, Windows capture and overlay, native clipboard, and diagnostics. Validate on the available display setup.
4. Record timings, execution coverage, and remaining risks. Stop for Product Owner review.

## Phase 1 recommendation

1. Agree on capture coverage and performance gates. Compare Windows Graphics Capture and Desktop Duplication only where Phase 0 evidence identifies a gap.
2. The capture UX preview delivers an in-memory native-to-editor handoff, pen, rectangle, arrow, undo/redo, and clipboard export. Follow up with selection handles, keyboard adjustment and PNG saving with a native save dialog.
3. Measure Canvas 2D on a representative 4K frame. Compare Konva or use Zustand if object transforms and immutability requirements make them useful.
4. Add object selection and the remaining shape, highlight and text tools, with Thai/English IME validation.
5. Validate offline install, DPI and monitor topology, memory and latency, and exports while the editor is active. Use shadcn/ui selectively for accessible editor and settings controls.
