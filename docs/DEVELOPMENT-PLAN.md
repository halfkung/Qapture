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

## Proposed 1.0 release criteria

| Priority | Deliverable | Acceptance before public release |
| --- | --- | --- |
| 1 | Complete the capture-to-export workflow | Resize a region, draw, correct a mark, copy to Paint, and save a PNG via a native file dialog. Confirm native dimensions and colors. Cancelling Save leaves the image editable. |
| 2 | Finish the foundation editor | Add the remaining foundation tools, including multiline Thai and English text. Verify IME composition, moving and resizing a mark, undo/redo and export consistency. |
| 3 | Qualify Windows and displays | Exercise Windows 10 and 11 on single and mixed-DPI monitors, including negative origins, gaps, 4K and 200% scaling. Run repeated captures with a disposable clipboard, measure the PRD latency and reliability targets, and record failures. |
| 4 | Distribute and upgrade | Sign the Windows binaries and installer, test clean install, upgrade from this demo with custom shortcuts preserved, uninstall, and offline WebView2 availability on fresh test machines. Publish versioned downloads and a short manual-update guide. |

For a one-minute demo, quit the previous tray instance, launch the standalone executable, choose Capture a region, drag, draw an arrow, undo and redo, then press Escape. To demonstrate clipboard export, choose Copy image explicitly and paste in Paint. Next, close the dashboard and use the displayed region shortcut to repeat from the tray. Local history, OCR and recording can follow this release.
