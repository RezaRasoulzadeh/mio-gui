# Render Testing

Rounded-rectangle tests use two independent paths:

- The GPU path renders the production WGSL shader into an offscreen `Rgba8Unorm` texture and reads its pixels through an aligned staging buffer.
- The CPU path supersamples the mathematical rounded-rectangle boundary and produces a reference alpha mask.

GPU tests must fail when an adapter, device, command submission, or buffer mapping operation fails. An unavailable renderer is not treated as a passing test.

GPU tests share one process-wide lock because some software adapters are unsafe under concurrent device setup.

The coverage matrix permits a maximum absolute alpha difference of `0.22` at any pixel and a mean absolute alpha difference of `0.001` across the render target. The measured worst case is a one-physical-pixel corner radius, with maximum error `0.21191406` and mean error `0.00050580647`. These tolerances account for the local difference between derivative-based analytic GPU coverage and true-area CPU supersampling. They must be remeasured if the rasterization model changes.

Horizontal and vertical reflection may differ by at most one 8-bit alpha unit. This verifies symmetry and equivalent four-corner coverage in the actual GPU output.

Fractional-boundary tests verify that the expanded instance quad preserves partial coverage outside the mathematical shape boundary while leaving pixels beyond its antialias margin untouched. Viewport clipping remains physical and may intentionally truncate coverage at the render-target edge.

Color comparisons will use per-channel tolerances defined alongside color-space golden tests. The current coverage test compares alpha only.

GPU matrix runs record adapter name, backend, device type, vendor and device IDs, driver, driver information, and measured worst cases in `target/mio-gui/render-tests/backend.txt`. Assertion failures include the same adapter identity. Cross-backend approval is required before the rounded-rectangle primitive is frozen.

Persistent CPU-reference golden images use ASCII PGM alpha masks so they remain dependency-free and reviewable as text. The committed radius matrix is regenerated only with `MIO_GUI_UPDATE_GOLDENS=1 cargo test rounded_rectangle_golden_images_match`.

Rectangle rendering uses GPU instancing over a read-only storage buffer. The current production batch capacity is 1,024 rectangles, and a batch is emitted through one draw call. Capacity growth and splitting policy will be revisited with renderer benchmarks.
