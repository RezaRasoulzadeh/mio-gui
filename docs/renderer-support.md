# Renderer Support

## Targets

Mio-GUI targets native Windows, Linux, and macOS applications without a browser or WebView.

The primary graphics backends are:

- Vulkan on Linux and supported Windows systems
- Metal on macOS
- Direct3D 12 on Windows

OpenGL is a compatibility fallback where wgpu exposes it. Browser WebGPU and WebGL are outside the current project scope.

## Adapter policy

Interactive windows request a high-performance adapter compatible with their surface. Renderer initialization must fail visibly when no compatible adapter or device is available. It must not silently switch to a behaviorally different software renderer in production.

Headless render tests request a low-power adapter without a surface. The test report records whether the selected adapter is discrete, integrated, virtual, or CPU-based. A software adapter validates shader behavior but does not replace hardware backend approval.

## Presentation

The current surface uses `AutoNoVsync` with one-frame maximum latency because this avoids a provisional-frame regression observed on macOS. Depending on backend support, this may select Immediate, Mailbox, or FIFO presentation. Tearing is therefore possible and presentation policy remains provisional until representative application workloads are measured.

## Recovery

Lost and outdated surfaces are reconfigured and retried once. Suboptimal frames are presented before reconfiguration. Timeouts and occlusion are recoverable skipped frames. Validation failures are reported as errors.

Device-loss recovery remains unfinished and must be tested before the renderer foundation is frozen.
