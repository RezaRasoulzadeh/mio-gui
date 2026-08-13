# Render Testing

Rounded-rectangle tests use two independent paths:

- The GPU path renders the production WGSL shader into an offscreen `Rgba8Unorm` texture and reads its pixels through an aligned staging buffer.
- The CPU path supersamples the mathematical rounded-rectangle boundary and produces a reference alpha mask.

GPU tests must fail when an adapter, device, command submission, or buffer mapping operation fails. An unavailable renderer is not treated as a passing test.

The initial coverage comparison permits a maximum absolute alpha difference of `0.15` at any pixel and a mean absolute alpha difference of `0.01` across the rectangle. These tolerances account for the difference between derivative-based analytic GPU coverage and the CPU supersampling reference. They must be tightened or justified with evidence if the rasterization model changes.

Color comparisons will use per-channel tolerances defined alongside color-space golden tests. The current coverage test compares alpha only.

Backend and adapter identity must be recorded when persistent golden images are introduced. Cross-backend approval is required before the rounded-rectangle primitive is frozen.
