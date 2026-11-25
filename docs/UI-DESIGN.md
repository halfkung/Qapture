# UI design

Use indigo `#6254E8` for primary actions and focus emphasis. Pair dark ink with cool light surfaces and use high-contrast equivalents in dark mode. Keep labels readable, controls keyboard accessible, and actions close to the screenshot.

| Surface | Responsibilities |
| --- | --- |
| System tray | Region capture, full desktop capture, settings entry, cancel, quit |
| Capture overlay | Frozen desktop, dim outside selection, physical pixel dimensions, release to edit, Escape to cancel |
| Quick annotation editor | Pen, rectangle, arrow, color and stroke controls, undo/redo, and copy |
| Dashboard | Capture entry points, recent operation status, access to shortcut settings |
| Settings | Shortcut bindings, conflict feedback, system appearance in Phase 0 |

The dashboard opens at 720 × 620 logical pixels. A window-and-selection illustration conveys the capture workflow. Two action cards link directly to region capture and full desktop copy. The header links to shortcut settings. Light and dark themes follow the system.

The shortcut list shows current bindings next to a labeled **Edit** button. The selected row focuses a keyboard recorder and displays a valid key combination as keycaps as soon as it is pressed. Validation appears next to the recorder. Saving or cancelling returns focus to **Edit**.

Phase 1 should preserve IME composition in multiline text. Prefer Segoe UI with Windows Thai font fallback, test common Thai diacritics and line layouts, and keep destructive actions explicit.
