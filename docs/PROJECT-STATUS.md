# Phase 0 status

The prototype is implemented with Tauri 2, React, TypeScript, Vite, Tailwind CSS, and Rust. It includes tray capture, a single-instance dashboard, persistent shortcuts, native desktop capture and region selection, and native-resolution clipboard export. The current milestone is Product Owner review before Phase 1.

On this workstation, the optimized executable is `.local/release-target/release/qapture.exe`. The unsigned x64 NSIS installer is `.local/release-target/release/bundle/nsis/Qapture_0.1.0_x64-setup.exe`. The executable launched and passed the desktop smoke test. Validation on a clean machine remains open.

## Dashboard refresh

The redesigned dashboard is available in the standalone preview at `.local/ui-refresh/Qapture.exe`. Its light and dark previews and the shortcut editor preview are in the same folder. The bundled installer contains the original Phase 0 dashboard.

| Test type | Results |
| --- | --- |
| Automated source checks | Frontend lint, five Vitest tests, production bundle, Rust formatting, Clippy, and six Rust tests passed. |
| Automated desktop checks | WebView2 validation covered the 720 × 600 dashboard, the compact 540 × 500 viewport, both themes, editing a shortcut, duplicate validation, and keyboard focus after save and Escape. |
| Visual review | Reviewed screenshots of both themes and the compact shortcut editor from the running app. |
| Unexecuted in this refresh | Native capture smoke, clipboard export, installer upgrade, and performance requalification. |

## Environment audit · 25 November 2026

The active checkout was empty and had no Git metadata. The path from the brief did not exist on this workstation. Work was initialized in the active checkout at `C:\Users\User\Documents\GitHub\Qapture`, with local Git on `main`.

| Dependency | Audit result |
| --- | --- |
| Windows | Windows 11 Home Single Language x64, build 26300 |
| Node.js | 22.12.0 |
| npm | 10.9.0 |
| Git | 2.47.1.windows.1 |
| WebView2 Runtime | 158.0.4370.33 |
| Rust, Cargo, Rustup | Absent on the system. Workspace-local Rust and Cargo 1.95.0 and Rustup 1.29.3 used for verification. |
| Visual Studio C++ Build Tools | No installed toolchain found. Microsoft compiler/linker files were extracted in the workspace. Compiler 19.44.35230, linker 14.44.35230. |
| Windows SDK | No installed SDK found. Workspace-local 10.0.26100 headers/import libraries and SDK BuildTools 10.0.26100.9457 were used. |

Toolchain binaries and caches are ignored under `.local`. No system-wide dependency installations or OS settings changes were required. Cargo 1.100.0 initially failed to start a build script on this host. Validation used 1.95.0 with retries for intermittent Windows process-launch errors. The original npm version's peer-dependency resolver failed on Vitest 4. Vitest 3.2.7 is locked for this prototype.

To reuse the toolchain already provisioned in this checkout, dot-source `.local/build-env.ps1` from the project root before running Cargo or the Tauri CLI. A fresh checkout can use the standard Windows prerequisites in the [README](../README.md).

## Validation

The desktop test fixture runs on the workstation's 1920 × 1200 display at 125%, with a negative X origin, and its 2560 × 1440 primary display at 100%. The fixture stores no screenshot files. The timings JSONL contains outcomes and durations only.

| Test type | Results |
| --- | --- |
| Automated source checks | Passed frontend lint, three Vitest tests, TypeScript/Vite production bundle, Rust formatting and Clippy, and six Rust tests. The Tauri development runner also loaded the React dashboard through Vite. |
| Automated desktop smoke | Passed release launch, close to tray, second-launch dashboard restoration, Windows hotkey reservation, frozen popup pixels, repeated-trigger protection, physical monitor bounds, mixed-DPI selection across the seam, native clipboard contention and cleanup in both capture modes, and 20 warm selection/cancel cycles |
| Dashboard interaction | Windows accessibility automation observed the conflict error when another process reserved the requested chord. Dashboard IPC tests confirmed the old shortcut stayed registered after a conflict, a successful update released it, and the new shortcut survived restart. |
| Manual checks | Tray menu clicks, visual checks on other Windows configurations, clipboard paste into Paint, and clean-machine install remain open. |
| Unexecuted | Successful native clipboard publication and its 1080p timing were skipped to preserve the user's clipboard. Windows 10, 4K, 200%, HDR, RDP, and a ≥99.9% reliability run also remain open. |

## Release benchmark

| Metric | Observed | Proposed target |
| --- | --- | --- |
| Warm overlay | 90.3 ms p95, 94.3 ms maximum, 20 samples | p95 ≤ 200 ms |
| Dashboard startup | 1,988.1 ms from process launch and 738.9 ms from Rust entry, one sample | ≤ 2 s |
| Idle tray CPU | 0.00% of host CPU rounded, measured over 4.3 s | ≤ 1% |
| Idle tray working set | 28.0 MiB across one associated process after dashboard close | ≤ 100 MB stretch |
| 1080p clipboard export | Pending successful publication and paste measurement | ≤ 300 ms |
| Controlled capture reliability | 20 of 20 warm region open/cancel cycles in this smoke run | ≥ 99.9% under a qualification test |

Overlay timing starts at the Qapture shortcut handler and stops after initial native paint and a compositor flush. Startup is a process-cold observation on an already running desktop. The test does not flush OS caches. Idle memory includes associated processes visible in the process tree. Sample sizes are suitable for a feasibility check. Repeat startup, idle, and reliability measurements across the qualification matrix.

## Phase 1 backlog

1. Review the prototype and agree on the Windows and display validation matrix.
2. Finish clipboard and installer acceptance, then add PNG export and resizable selection.
3. Prototype a non-destructive Konva editor on a native-size 4K image and test the projected WebView memory cost. Establish object transforms, command history, and Thai IME behavior before adding all tools.
4. Qualify idle and editor-active performance, with Windows Graphics Capture or Desktop Duplication experiments if the GDI backend misses agreed coverage.

See the [known issues](KNOWN-ISSUES.md), [test plan](TEST-PLAN.md), and [development plan](DEVELOPMENT-PLAN.md).
