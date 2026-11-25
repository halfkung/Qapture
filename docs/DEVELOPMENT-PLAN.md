# Development plan

## Phase 0

1. Audit the Windows workstation and resolve development prerequisites locally where practical.
2. Initialize Git, documentation, frontend tooling, and the Tauri shell. Build and launch the shell.
3. Implement the native tray and settings, Windows capture and overlay, native clipboard, and diagnostics. Validate on the available display setup.
4. Record timings, execution coverage, and remaining risks. Stop for Product Owner review.

## Phase 1 recommendation

1. Agree on capture coverage and performance gates. Compare Windows Graphics Capture and Desktop Duplication only where Phase 0 evidence identifies a gap.
2. Deliver selection handles, keyboard adjustment, PNG saving with a native save dialog, and a documented native-to-editor image handoff.
3. Prototype Konva on a representative 4K frame with physical pixel transforms. Use Zustand for editor state if it simplifies immutable commands, history, and tool state.
4. Build object selection and undo/redo alongside the initial rectangle, ellipse, line, and arrow tools. Add freehand, highlighter, and text with Thai/English IME validation.
5. Validate offline install, DPI and monitor topology, memory and latency, and exports while the editor is active. Use shadcn/ui selectively for accessible editor and settings controls.
