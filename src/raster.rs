// raster.rs

#[derive(Clone, Copy)]
pub struct RoundedRectMask {
    size: [f32; 2],
    radius: f32,
}

impl RoundedRectMask {
    pub fn new(size: [f32; 2], radius: f32) -> Self {
        let size = [size[0].max(0.0), size[1].max(0.0)];
        let radius = radius.max(0.0).min(size[0].min(size[1]) * 0.5);

        Self { size, radius }
    }

    pub fn size(self) -> [f32; 2] {
        self.size
    }

    pub fn radius(self) -> f32 {
        self.radius
    }

    pub fn signed_distance(self, point: [f32; 2]) -> f32 {
        let half_size = [self.size[0] * 0.5, self.size[1] * 0.5];
        let centered = [point[0] - half_size[0], point[1] - half_size[1]];
        let q = [
            centered[0].abs() - half_size[0] + self.radius,
            centered[1].abs() - half_size[1] + self.radius,
        ];
        let outside = [q[0].max(0.0), q[1].max(0.0)];

        outside[0].hypot(outside[1]) + q[0].max(q[1]).min(0.0) - self.radius
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

#[cfg(test)]
mod tests {
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
}
