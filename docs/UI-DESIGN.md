# UI design

Use indigo `#6254E8` for primary actions and focus emphasis. Pair dark ink with cool light surfaces and use high-contrast equivalents in dark mode. Keep labels readable, controls keyboard accessible, and actions close to the screenshot.

| Surface | Responsibilities |
| --- | --- |
| System tray | Region capture, full desktop capture, settings entry, cancel, quit |
| Capture overlay | Frozen desktop, dim outside selection, physical pixel dimensions, keyboard copy/cancel |
| Quick annotation editor | Phase 1 floating toolbar, object selection and transforms, property controls, copy and save |
| Dashboard | Capture entry points, recent operation status, access to shortcut settings |
| Settings | Shortcut bindings, conflict feedback, system appearance in Phase 0 |

Phase 1 should preserve IME composition in multiline text. Prefer Segoe UI with Windows Thai font fallback, test common Thai diacritics and line layouts, and keep destructive actions explicit.
