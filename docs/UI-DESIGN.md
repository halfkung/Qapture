# UI design

Use indigo `#6254E8` for primary actions and focus emphasis. Pair dark ink with cool light surfaces and use high-contrast equivalents in dark mode. Keep labels readable, controls keyboard accessible, and actions close to the screenshot.

| Surface | Responsibilities |
| --- | --- |
| System tray | Region, full desktop, window capture, History, dashboard, cancel and quit |
| Capture overlay | Frozen desktop, dim outside selection, dimensions, window selection, magnifier and optional move/resize |
| Quick annotation editor | Canvas tools, contextual properties, object actions, zoom, history, copy, Save and pin |
| Dashboard | Capture entry points, delay, region options, recent status, shortcut settings and History navigation |
| History and settings | Project search, reopen and deletion, import, storage preferences and startup toggle |

The dashboard opens at 920 × 740 logical pixels, bounded by the monitor work area. Three action cards link to region capture, full desktop copy and window capture. Light and dark themes follow the system.

The default dashboard fits at both the opening size and the 540 × 500 minimum. The header stays in place when expanded settings or errors require the content pane to scroll. The pane keeps its scrollbar hidden and can still be scrolled by mouse, touch, or keyboard focus.

The shortcut list shows current bindings next to a labeled **Edit** button. The selected row focuses a keyboard recorder and displays a valid key combination as keycaps as soon as it is pressed. Validation appears next to the recorder. Saving or cancelling returns focus to **Edit**.

The editor uses a multiline text control with Segoe UI and Windows Thai font fallback for the canvas. It suspends shortcut actions during focused text entry. History deletion requires confirmation and storage settings explain that editable projects include original pixels.
