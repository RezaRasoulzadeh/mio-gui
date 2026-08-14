# Direction-aware layout

## Direction cascade

Every layout subtree has a resolved `Direction::Ltr` or `Direction::Rtl`. A node chooses `DirectionSetting::Inherit`, `Ltr`, or `Rtl`; inheritance reads only the resolved direction of its parent. An override becomes the inherited value for descendants until another override appears.

Direction changes inline semantics, not the physical coordinate system. X still increases toward the physical right and Y still increases downward.

## Logical edges

`FlowEdges` stores `block_start`, `inline_end`, `block_end`, and `inline_start`. With horizontal writing:

- Block start is physical top in both directions.
- Block end is physical bottom in both directions.
- Inline start is left in LTR and right in RTL.
- Inline end is right in LTR and left in RTL.

Padding and margin should use `FlowEdges` unless their semantics are intentionally physical. `LogicalEdges` remains available for explicit top/right/bottom/left behavior and is never mirrored automatically.

## Alignment

`InlineAlignment::Start` and `End` resolve through the local direction. `Center` is direction-independent. Alignment offsets clamp remaining space to zero, so an oversized child does not receive a negative position from alignment alone.
