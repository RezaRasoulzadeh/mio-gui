# Render Testing

Rounded-rectangle tests use two independent paths:

- The GPU path renders the production WGSL shader into an offscreen `Rgba8Unorm` texture and reads its pixels through an aligned staging buffer.
- The CPU path supersamples the mathematical rounded-rectangle boundary and produces a reference alpha mask.

GPU tests must fail when an adapter, device, command submission, or buffer mapping operation fails. An unavailable renderer is not treated as a passing test.

The coverage matrix permits a maximum absolute alpha difference of `0.22` at any pixel and a mean absolute alpha difference of `0.001` across the render target. The measured worst case is a one-physical-pixel corner radius, with maximum error `0.21191406` and mean error `0.00050580647`. These tolerances account for the local difference between derivative-based analytic GPU coverage and true-area CPU supersampling. They must be remeasured if the rasterization model changes.

Horizontal and vertical reflection may differ by at most one 8-bit alpha unit. This verifies symmetry and equivalent four-corner coverage in the actual GPU output.

Color comparisons will use per-channel tolerances defined alongside color-space golden tests. The current coverage test compares alpha only.

Backend and adapter identity must be recorded when persistent golden images are introduced. Cross-backend approval is required before the rounded-rectangle primitive is frozen.
