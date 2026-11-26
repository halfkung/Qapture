# Project status

## 0.2.0 Windows preview

Run the standalone `.local/release-0.2.0/Qapture.exe`, or quit older Qapture versions from the tray and run `.local/release-0.2.0/Qapture_0.2.0_x64-setup.exe` to install. The preview keeps the existing application identifier. The installer is unsigned.

| Verification | Current result |
| --- | --- |
| Automated source and packaging | Frontend lint, 12 frontend tests, production bundle, Rust formatting, Clippy with warnings denied, 15 Rust tests, and optimized x64 NSIS packaging passed. |
| Windows fixture | Verified delayed window capture at 720 × 460, adjusted region at 240 × 140, image import, Thai/English entry and text movement, undo/redo, crop, clipboard contention, pin creation, opt-in project retention, search, reopen, deletion, configured PNG export and automatic PNG/JPG/WebP export. Independently decoded the exported PNG and JPG dimensions. The PNG redaction sample was opaque. |
| Optimized desktop smoke | Passed physical monitor bounds, dimmed background, overlapping capture protection, frozen popup pixels with configured non-Alt keys, mixed-DPI monitor seam, full-desktop clipboard contention, shortcut reservation, close to tray and second-launch dashboard restoration. |
| Manual | Reviewed the synthetic editor screenshot for Thai tone marks, multiline text and crop presentation. A native Save As operation returned successfully and the output PNG decoded as 500 × 300. |
| Pending | Successful external paste, physical Thai IME input, sleep/wake, clean install/upgrade/uninstall, sign-in startup, Windows 10/4K/200% regression and the statistical reliability target. The clipboard was preserved. |

An optimized isolated build used the Windows 11 workstation's 2560 × 1440 display at 100% and 1920 × 1200 display at 125%. The second display has a negative X origin.

| Metric | Observed | Target |
| --- | --- | --- |
| Warm hotkey to overlay | 75.9 ms p95 and 100.4 ms maximum, 20 samples | p95 ≤ 200 ms |
| Warm open and cancel | 20 of 20 returned to idle | ≥99.9% in a qualification run |
| Dashboard startup | 1,544.4 ms from launch, one sample | ≤ 2 s |
| Idle tray CPU and memory | 0.00% host CPU rounded over 4.2 s, 29.7 MiB combined working set | ≤ 1% CPU, ≤ 100 MB stretch target |
| Successful 1080p clipboard export | Unmeasured | ≤ 300 ms |

See [Feature audit](FEATURE-AUDIT.md) for the baseline, implementation and advanced backlog.

## Project foundations

The prototype is implemented with Tauri 2, React, TypeScript, Vite, Tailwind CSS, and Rust. It includes tray capture, a single-instance dashboard, persistent shortcuts, native desktop capture and region selection, and native-resolution clipboard export. After the Phase 0 review, the capture UX preview added the initial quick editor.

## 0.1.2 Windows demo

Use `.local/release-0.1.2/Qapture.exe` to try the demo. Use `.local/release-0.1.2/Qapture_0.1.2_x64-setup.exe` to install. Quit earlier copies from the tray before launching. The installer uses the same application identifier and settings location.

| Verification | Result |
| --- | --- |
| Automated source checks | Frontend lint, nine frontend tests, production build, Rust formatting, Clippy with warnings denied, and thirteen Rust tests passed. |
| Automated Windows UI | An isolated build opened at 920 × 740. With two real Win32 hotkey conflicts, the idle dashboard and lower recorder had zero page and pane overflow in both themes at 920 × 740, 720 × 620 and 540 × 500. Recovery registered an available alternative for one action, restored the original for the other after its fixture released it, and both registered again after restart. |
| Manual review | Reviewed the Windows dashboard screenshot for spacing, readability and visibility of both settings rows. |
| Packaged executable | Verified both delivered files match their optimized build outputs and carry version 0.1.2. Launched the standalone executable and confirmed the Qapture dashboard window opened. |
| Unexecuted | In-place installation, clean-machine installation and successful paste into a second application. The existing running Qapture and clipboard were preserved. |

The proposed [1.0 release criteria](DEVELOPMENT-PLAN.md#proposed-10-release-criteria) cover export, editing, Windows qualification, and distribution.

## 0.1.1 release

The standalone Windows executable is `.local/release-0.1.1/Qapture.exe`. The unsigned x64 NSIS installer is `.local/release-0.1.1/Qapture_0.1.1_x64-setup.exe`. The installer retains the original application identifier and settings location.

The dashboard shows illustrated capture controls and records shortcuts from key presses. Its idle content fits at the default and minimum sizes without scrollbars. Region capture dims the surrounding desktop and opens the editor on release. The editor provides pen, rectangle, arrow, undo, redo and clipboard copy.

| Test type | Results |
| --- | --- |
| Automated source checks | Frontend lint, nine Vitest tests, production bundle, Rust formatting, Clippy, and ten Rust tests passed. |
| Dashboard 0.1.1 | An isolated Windows build verified both themes at 720 × 620 and 540 × 500. Every idle control fit without scrolling. Expanding the lower shortcut form scrolled the content while the header stayed visible and no scrollbar appeared. |
| Capture workflow | The preceding isolated preview verified key recording and persistence, dimming and unchanged selected pixels, editor handoff, native dimensions and colors across the monitor seam, drawing and undo/redo, protected clipboard contention, and the return to the dashboard. |
| Unexecuted | Successful copy/paste, updated capture benchmarks, an in-place installer upgrade, and clean-machine installation. The existing installation and clipboard were left intact during testing. |

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

## Phase 0 release benchmark

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
