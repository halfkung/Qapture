# Working on Qapture

- Keep changes inside Qapture and preserve work already in the checkout.
- Target Windows 10/11 with Tauri 2, React, TypeScript, Vite, and Rust. Document findings and get the Product Owner’s approval before changing the primary stack.
- Keep capture, windows, shortcuts, tray, state, clipboard, export, settings, storage, and diagnostics modular.
- Use signed physical pixel coordinates for desktop positions and physical pixels for image sizes. Test negative origins, monitor gaps, and mixed scaling.
- Keep captured pixels in local memory unless the user explicitly copies or saves. Never log or upload image data. Keep Tauri capabilities scoped to the surfaces that use them.
- Write meaningful regression tests. Run frontend lint, tests, and production build plus Rust formatting, Clippy, and tests for affected changes. Record automated, manual, and unexecuted validation separately.
- Update technical decisions, known issues, and project status when behavior or verification changes. Review changes before local Git checkpoints. Do not create or push a remote without authorization.
- Complete and validate core capture, editing and export before qualifying productivity and advanced features. Track unresolved release gates in the feature audit.
