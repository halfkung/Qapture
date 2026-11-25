# Known issues and risks

- The GDI backend can omit content excluded by Windows, DRM, or GPU overlay mechanisms. HDR color fidelity and remote desktop behavior are unverified.
- The Phase 0 prototype will need testing across Windows 10, 4K, 200% scaling, portrait displays, and graphics driver combinations.
- The native overlay stays on top while a selection is active. A system menu can delay keyboard focus. Click the overlay to resume keyboard interaction or right-click to cancel.
- The native overlay bounds the virtual desktop to 64 megapixels and limits each selection dimension to fewer than 32,767 pixels.
- The main window releases WebView2 on close. Editor-active memory and the startup distribution still need qualification.
- The 0.1.1 installer is unsigned. Validate installation, upgrade, removal, offline runtime prerequisites, and signing on the release matrix.
- Successful copy and paste to independent applications and the 1080p clipboard target remain unverified on the kickoff workstation.
- Windows or another application can intercept reserved shortcuts before they reach the recorder.
- Quick editing provides pen, rectangle and arrow markup. Text, individual object transforms, and PNG save remain on the roadmap.
