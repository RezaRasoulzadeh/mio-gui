# Window Resize Behavior

Mio-GUI records the latest non-zero size when `winit` delivers a `Resized` event and requests a new frame. It does not synchronously reconfigure the surface from the event handler.

Immediately before the next frame is acquired, the renderer consumes the latest pending size and configures the surface once. A burst of queued resize events therefore collapses to the newest dimensions available at the render boundary rather than forcing repeated swapchain recreation on the event-loop callback path.

On the tested Ubuntu configuration, maximizing from `800 × 600` to `1870 × 1013` produced one final-size event rather than a sequence of intermediate sizes. The surface was configured within approximately one millisecond and a correctly sized frame was presented approximately three milliseconds after the event.

Any scaling visible before that event belongs to the window manager's maximize transition. The application cannot render intermediate layouts when the platform does not expose intermediate content sizes. Continuous redraws do not correct this condition because every pre-event frame necessarily has the old dimensions.

The surface uses `AutoNoVsync` with one-frame maximum latency. This configuration avoids an initial presentation regression observed on macOS and selects Immediate, Mailbox, or FIFO according to backend support. Tearing remains a platform-dependent possibility when Immediate presentation is selected.

Normal interactive resizing may produce a stream of size events. Mio-GUI retains only the newest pending size and schedules rendering without blocking the event handler on surface configuration or presentation.

Surface creation and genuine size or scale-factor changes schedule three frames. Duplicate events do not restart this sequence. This replaces provisional drawables observed during macOS startup and resize without entering a continuous redraw loop.

Resize diagnostics can be enabled with `MIO_GUI_DIAGNOSTICS=1`. They record resize events, surface configuration, acquired texture dimensions, and presentation timing.
