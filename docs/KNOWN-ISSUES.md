# Known issues and risks

- The GDI backend can omit content excluded by Windows, DRM, or GPU overlay mechanisms. HDR color fidelity and remote desktop behavior are unverified.
- The Phase 0 prototype will need testing across Windows 10, 4K, 200% scaling, portrait displays, and graphics driver combinations.
- WebView2 contributes to startup and idle memory. Measure it with the dashboard visible and hidden.
- The full annotation editor and PNG save belong to Phase 1.
