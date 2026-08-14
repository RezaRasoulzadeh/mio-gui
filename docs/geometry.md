# Geometry and Pixel Conventions

## Coordinate spaces

Public layout and component geometry uses logical pixels. Logical coordinates are independent of monitor density and are converted to physical pixels at the renderer boundary.

The renderer uses physical pixels for surface dimensions, GPU geometry, signed-distance calculations, corner radii, borders, and antialiasing. A logical value is converted by multiplying it by the active window scale factor.

The origin is at the top-left of the content area. X increases toward the physical right and Y increases downward. Rendering geometry is direction-neutral. RTL and LTR layout resolve logical start and end positions before geometry reaches the renderer.

Logical and physical values use different Rust coordinate-space types. A validated, finite, positive `ScaleFactor` is required to cross that boundary. Conversion preserves fractional coordinates in both directions; layout does not snap merely because a value reaches the renderer.

## Pixel rounding

Analytic drawing uses unsnapped physical floating-point geometry so fractional positions, sizes, radii, and borders retain correct antialias coverage.

Operations that require integral pixel boundaries choose an explicit `PixelSnap` policy:

- `Nearest` rounds each rectangle boundary independently to its nearest physical pixel.
- `Outward` floors the minimum boundaries and ceils the maximum boundaries. Clip and scissor allocation uses this policy so covered pixels are never discarded.
- `Inward` ceils the minimum boundaries and floors the maximum boundaries. This is only for callers that require pixels wholly contained by a rectangle.

Rounding boundaries instead of independently rounding origin and size prevents accumulated edge disagreement between adjacent rectangles. Hit-testing uses unsnapped logical geometry.

## Overflow and clipping

`Overflow::Visible` produces an unbounded clip region. `Overflow::Clip` produces the element's rectangular bounds, or an empty region when either bound dimension is zero.

Nested clips intersect immediately through `ClipStack`. Empty regions remain empty, unbounded regions adopt the other operand, and disjoint rectangles become empty. Popping restores the exact parent clip and cannot remove the unbounded root.

Clip regions follow affine transforms by taking the axis-aligned bounding box of all four transformed corners. At the GPU boundary, a physical clip intersects the viewport first and then rounds outward to an integer `PhysicalPixelRect`; wholly offscreen and zero-area clips produce no scissor rectangle.

## Rectangle boundaries

A rectangle is represented by a top-left position and a non-negative size. Position denotes the outer boundary rather than the center of a pixel. The opposite boundary is `position + size`.

Layout containment is half-open: the start edges are included and the end edges are excluded. A point is inside when `x >= start_x`, `x < end_x`, `y >= start_y`, and `y < end_y`. Raster coverage remains continuous around the mathematical boundary and is not a binary containment test.

Layout rectangles are not reduced to fit the viewport. Geometry outside the viewport remains valid and is clipped during rendering.

Negative width, height, and radius inputs are normalized to zero. A corner radius is limited to half of the rectangle's shortest side.

Independent corner radii use the physical order top-left, top-right, bottom-right, bottom-left. The renderer is direction-neutral; higher layout and component layers resolve logical start/end corners into this physical order.

When adjacent radii would overlap, all four radii are reduced by one common scale factor until every horizontal and vertical pair fits its side. Preserving their proportions prevents abrupt changes between differently rounded corners.

Borders are drawn inward from the rectangle's outer boundary and do not affect layout size or outer corner geometry. Negative border widths become zero. Border width is limited to half of the shortest side; at that limit the border consumes the entire interior.

## Rasterization

The rounded-rectangle distance field describes the exact requested physical size. Antialias coverage must not reduce that size.

The GPU quad extends beyond the mathematical rectangle by a small physical-pixel margin. This provides fragments outside the boundary so analytic antialiasing is not cut off by triangle rasterization.

Antialias width is calculated from screen-space derivatives of the signed distance. The coverage transition is centered on the mathematical boundary.

## Color

Shader colors are linear floating-point RGBA values. An sRGB surface performs the final linear-to-sRGB encoding. Shape alpha uses straight-alpha blending.

## Invariants requiring automated verification

- Logical dimensions remain constant when scale factor changes.
- Physical dimensions equal logical dimensions multiplied by scale factor.
- Requested geometry does not change when it crosses the viewport boundary.
- Rounded corners remain circular in physical pixels.
- All corners have equivalent coverage under reflection.
- CPU and WGSL uniform layouts remain byte-compatible.
