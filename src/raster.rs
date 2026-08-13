// raster.rs

#[derive(Clone, Copy)]
#[cfg(test)]
pub struct RoundedRectMask {
    size: [f32; 2],
    radii: [f32; 4],
}

#[cfg(test)]
impl RoundedRectMask {
    pub fn new(size: [f32; 2], radius: f32) -> Self {
        Self::with_radii(size, [radius; 4])
    }

    pub fn with_radii(size: [f32; 2], radii: [f32; 4]) -> Self {
        let size = [size[0].max(0.0), size[1].max(0.0)];
        let radii = normalize_corner_radii(size, radii);

        Self { size, radii }
    }

    pub fn size(self) -> [f32; 2] {
        self.size
    }

    pub fn radius(self) -> f32 {
        self.radii[0]
    }

    pub fn radii(self) -> [f32; 4] {
        self.radii
    }

    pub fn signed_distance(self, point: [f32; 2]) -> f32 {
        let half_size = [self.size[0] * 0.5, self.size[1] * 0.5];
        let centered = [point[0] - half_size[0], point[1] - half_size[1]];
        let radius = match (centered[0] > 0.0, centered[1] > 0.0) {
            (false, false) => self.radii[0],
            (true, false) => self.radii[1],
            (true, true) => self.radii[2],
            (false, true) => self.radii[3],
        };
        let q = [
            centered[0].abs() - half_size[0] + radius,
            centered[1].abs() - half_size[1] + radius,
        ];
        let outside = [q[0].max(0.0), q[1].max(0.0)];

        outside[0].hypot(outside[1]) + q[0].max(q[1]).min(0.0) - radius
    }

    pub fn coverage(self, pixel: [u32; 2], samples_per_axis: u32) -> f32 {
        self.coverage_at(pixel, [0.0, 0.0], samples_per_axis)
    }

    pub fn coverage_at(self, pixel: [u32; 2], origin: [f32; 2], samples_per_axis: u32) -> f32 {
        assert!(samples_per_axis > 0);

        let mut covered = 0_u64;
        for sample_y in 0..samples_per_axis {
            for sample_x in 0..samples_per_axis {
                let point = [
                    pixel[0] as f32 + (sample_x as f32 + 0.5) / samples_per_axis as f32 - origin[0],
                    pixel[1] as f32 + (sample_y as f32 + 0.5) / samples_per_axis as f32 - origin[1],
                ];
                if self.signed_distance(point) <= 0.0 {
                    covered += 1;
                }
            }
        }

        let sample_count = u64::from(samples_per_axis) * u64::from(samples_per_axis);
        covered as f32 / sample_count as f32
    }

    pub fn rasterize(self, dimensions: [u32; 2], samples_per_axis: u32) -> Vec<f32> {
        self.rasterize_at(dimensions, [0.0, 0.0], samples_per_axis)
    }

    pub fn rasterize_at(
        self,
        dimensions: [u32; 2],
        origin: [f32; 2],
        samples_per_axis: u32,
    ) -> Vec<f32> {
        let mut mask =
            Vec::with_capacity((dimensions[0] as usize).saturating_mul(dimensions[1] as usize));

        for y in 0..dimensions[1] {
            for x in 0..dimensions[0] {
                mask.push(self.coverage_at([x, y], origin, samples_per_axis));
            }
        }

        mask
    }
}

pub fn normalize_corner_radii(size: [f32; 2], radii: [f32; 4]) -> [f32; 4] {
    let size = [size[0].max(0.0), size[1].max(0.0)];
    let radii = radii.map(|radius| radius.max(0.0));
    let ratios = [
        side_ratio(size[0], radii[0] + radii[1]),
        side_ratio(size[0], radii[3] + radii[2]),
        side_ratio(size[1], radii[0] + radii[3]),
        side_ratio(size[1], radii[1] + radii[2]),
    ];
    let scale = ratios.into_iter().fold(1.0_f32, f32::min);
    radii.map(|radius| radius * scale)
}

fn side_ratio(length: f32, radius_sum: f32) -> f32 {
    if radius_sum > 0.0 {
        length / radius_sum
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::RoundedRectMask;

    fn assert_approximately_equal(left: f32, right: f32) {
        assert!((left - right).abs() <= f32::EPSILON);
    }

    #[test]
    fn clamps_radius_to_shortest_half_side() {
        let mask = RoundedRectMask::new([40.0, 18.0], 20.0);

        assert_eq!(mask.radius(), 9.0);
    }

    #[test]
    fn clamps_negative_geometry_to_zero() {
        let mask = RoundedRectMask::new([-40.0, -18.0], -5.0);

        assert_eq!(mask.size(), [0.0, 0.0]);
        assert_eq!(mask.radius(), 0.0);
    }

    #[test]
    fn distance_is_symmetric_across_both_axes() {
        let mask = RoundedRectMask::new([40.0, 18.0], 4.5);
        let points = [[2.25, 1.75], [10.5, 4.25], [19.5, 8.5]];

        for point in points {
            let reflected_x = [mask.size()[0] - point[0], point[1]];
            let reflected_y = [point[0], mask.size()[1] - point[1]];
            let reflected_xy = [reflected_x[0], reflected_y[1]];
            let distance = mask.signed_distance(point);

            assert_approximately_equal(distance, mask.signed_distance(reflected_x));
            assert_approximately_equal(distance, mask.signed_distance(reflected_y));
            assert_approximately_equal(distance, mask.signed_distance(reflected_xy));
        }
    }

    #[test]
    fn all_corner_coverages_are_identical() {
        let dimensions = [40, 18];
        let mask = RoundedRectMask::new([dimensions[0] as f32, dimensions[1] as f32], 4.5);
        let pixels = [
            [0, 0],
            [dimensions[0] - 1, 0],
            [0, dimensions[1] - 1],
            [dimensions[0] - 1, dimensions[1] - 1],
        ];
        let coverage = mask.coverage(pixels[0], 16);

        for pixel in pixels {
            assert_approximately_equal(coverage, mask.coverage(pixel, 16));
        }
    }

    #[test]
    fn raster_is_symmetric_pixel_by_pixel() {
        let dimensions = [40, 18];
        let mask = RoundedRectMask::new([dimensions[0] as f32, dimensions[1] as f32], 4.5)
            .rasterize(dimensions, 8);

        for y in 0..dimensions[1] {
            for x in 0..dimensions[0] {
                let index = (y * dimensions[0] + x) as usize;
                let reflected_x = (y * dimensions[0] + dimensions[0] - x - 1) as usize;
                let reflected_y = ((dimensions[1] - y - 1) * dimensions[0] + x) as usize;

                assert_approximately_equal(mask[index], mask[reflected_x]);
                assert_approximately_equal(mask[index], mask[reflected_y]);
            }
        }
    }

    #[test]
    fn fractional_scaled_geometry_preserves_proportions() {
        let scale_factor = 1.25;
        let mask = RoundedRectMask::new(
            [40.0 * scale_factor, 18.0 * scale_factor],
            4.5 * scale_factor,
        );

        assert_eq!(mask.size(), [50.0, 22.5]);
        assert_eq!(mask.radius(), 5.625);
        assert!(mask.signed_distance([25.0, 11.25]) < 0.0);
    }

    #[test]
    fn fractional_origin_moves_coverage_without_changing_shape() {
        let mask = RoundedRectMask::new([10.0, 6.0], 2.0);
        let unshifted = mask.rasterize_at([16, 12], [2.0, 2.0], 8);
        let shifted = mask.rasterize_at([16, 12], [2.5, 2.5], 8);

        assert_ne!(unshifted, shifted);
        assert_eq!(
            mask.coverage_at([2, 2], [2.5, 2.5], 8),
            mask.coverage_at([12, 2], [2.5, 2.5], 8),
        );
    }

    #[test]
    fn preserves_independent_corner_radii() {
        let mask = RoundedRectMask::with_radii([40.0, 20.0], [2.0, 5.0, 8.0, 0.0]);

        assert_eq!(mask.radii(), [2.0, 5.0, 8.0, 0.0]);
        assert_ne!(
            mask.signed_distance([0.5, 0.5]),
            mask.signed_distance([39.5, 0.5]),
        );
    }

    #[test]
    fn proportionally_clamps_overlapping_corner_radii() {
        let mask = RoundedRectMask::with_radii([40.0, 20.0], [20.0, 30.0, 10.0, 10.0]);

        assert_eq!(mask.radii(), [10.0, 15.0, 5.0, 5.0]);
    }

    fn pgm(mask: &[f32], dimensions: [u32; 2]) -> String {
        let mut output = format!("P2\n{} {}\n255\n", dimensions[0], dimensions[1]);
        for row in mask.chunks(dimensions[0] as usize) {
            for (index, alpha) in row.iter().enumerate() {
                if index > 0 {
                    output.push(' ');
                }
                output.push_str(&(alpha * 255.0).round().to_string());
            }
            output.push('\n');
        }
        output
    }

    #[test]
    fn rounded_rectangle_golden_images_match() {
        let dimensions = [32, 16];
        let cases = [
            ("zero-radius.pgm", 0.0),
            ("small-radius.pgm", 1.0),
            ("medium-radius.pgm", 3.0),
            ("maximum-radius.pgm", 6.0),
            ("oversized-radius.pgm", 100.0),
        ];
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/goldens");
        let update = std::env::var_os("MIO_GUI_UPDATE_GOLDENS").is_some();

        if update {
            std::fs::create_dir_all(&directory).unwrap();
        }

        for (filename, radius) in cases {
            let mask =
                RoundedRectMask::new([24.0, 12.0], radius).rasterize_at(dimensions, [4.0, 2.0], 32);
            let actual = pgm(&mask, dimensions);
            let path = directory.join(filename);

            if update {
                std::fs::write(&path, &actual).unwrap();
            }

            let expected = std::fs::read_to_string(&path).unwrap_or_else(|error| {
                panic!(
                    "{}: {error}; run MIO_GUI_UPDATE_GOLDENS=1 cargo test rounded_rectangle_golden_images_match",
                    path.display(),
                )
            });
            assert_eq!(actual, expected, "{}", path.display());
        }
    }
}
