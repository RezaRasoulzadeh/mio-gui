# Rounded Rectangle Golden Images

These ASCII PGM files contain deterministic CPU-reference alpha coverage at 32 samples per pixel axis. They cover zero, small, medium, maximum, and oversized corner radii.

Regenerate them only after deliberately reviewing a geometry or rasterization contract change:

```bash
MIO_GUI_UPDATE_GOLDENS=1 cargo test rounded_rectangle_golden_images_match
```

GPU output is compared separately with documented tolerances because exact rasterization bytes can vary by graphics backend and adapter.
