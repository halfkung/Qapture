# Known issues and risks

- The GDI backend can omit content excluded by Windows, DRM, or GPU overlay mechanisms. HDR color fidelity and remote desktop behavior are unverified.
- The Phase 0 prototype will need testing across Windows 10, 4K, 200% scaling, portrait displays, and graphics driver combinations.
- The native overlay stays on top while a selection is active. A system menu can delay keyboard focus. Click the overlay to resume keyboard interaction or right-click to cancel.
- The native overlay bounds the virtual desktop to 64 megapixels and limits each selection dimension to fewer than 32,767 pixels.
- The main window releases WebView2 on close. Editor-active memory and the startup distribution still need qualification.
- The 0.1.2 demo installer is unsigned. Installation on a clean machine, upgrade, removal, and offline WebView2 prerequisites still need release qualification.
- Successful copy and paste to independent applications and the 1080p clipboard target remain unverified on the kickoff workstation.
- Windows or another application can intercept reserved shortcuts before they reach the recorder.
- Windows reports a shortcut conflict without naming its owner. Check for an earlier Qapture in the tray. If all suggested keys are occupied, use Edit to record a different key or capture from the dashboard.
- Window capture selects a visible desktop rectangle. Occluding windows, shadows and other composed pixels can appear. Minimized or hidden windows cannot be reconstructed by this path.
- The Windows menu system can consume Alt before capture and dismiss a popup. Use a non-Alt shortcut or delayed dashboard capture for menus.
- A saved editable project retains original pixels. Solid redaction protects the flattened export. Blur and mosaic are visual effects.
- Start with Windows uses the executable path that was enabled. Re-enable the setting if you move the standalone executable. Installer removal of the startup entry needs qualification.
- Large captures and full-frame annotation effects need memory and responsiveness verification at 4K. Mixed-DPI scaling and clean installer validation are tracked in the test plan.
- The Windows Save As dialog can be slow to appear on the test workstation. Configuring an export folder enables direct Save and keeps Save As available for choosing another location.
