// geometry.rs

use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Logical;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Physical;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point<Unit> {
    pub x: f32,
    pub y: f32,
    unit: PhantomData<Unit>,
}

impl<Unit> Point<Unit> {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: finite_signed(x),
            y: finite_signed(y),
            unit: PhantomData,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaleFactor(f32);

impl ScaleFactor {
    pub fn new(value: f32) -> Option<Self> {
        (value.is_finite() && value > 0.0).then_some(Self(value))
    }

    pub fn new_or_one(value: f32) -> Self {
        Self::new(value).unwrap_or_default()
    }

    pub const fn get(self) -> f32 {
        self.0
    }

    pub fn logical_to_physical(self) -> Transform<Logical, Physical> {
        Transform::from_matrix([self.0, 0.0, 0.0, self.0, 0.0, 0.0])
    }

    pub fn physical_to_logical(self) -> Transform<Physical, Logical> {
        let reciprocal = self.0.recip();
        Transform::from_matrix([reciprocal, 0.0, 0.0, reciprocal, 0.0, 0.0])
    }
}

impl Default for ScaleFactor {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum PixelSnap {
    #[default]
    Nearest,
    Outward,
    Inward,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Overflow {
    #[default]
    Visible,
    Clip,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ClipRegion<Unit> {
    #[default]
    Unbounded,
    Empty,
    Rect(Rect<Unit>),
}

impl<Unit: Copy> ClipRegion<Unit> {
    pub fn from_overflow(bounds: Rect<Unit>, overflow: Overflow) -> Self {
        match overflow {
            Overflow::Visible => Self::Unbounded,
            Overflow::Clip if bounds.size.is_empty() => Self::Empty,
            Overflow::Clip => Self::Rect(bounds),
        }
    }

    pub fn intersect(self, other: Self) -> Self {
        match (self, other) {
            (Self::Empty, _) | (_, Self::Empty) => Self::Empty,
            (Self::Unbounded, region) | (region, Self::Unbounded) => region,
            (Self::Rect(left), Self::Rect(right)) => left
                .intersection(right)
                .map(Self::Rect)
                .unwrap_or(Self::Empty),
        }
    }

    pub fn contains(self, point: Point<Unit>) -> bool {
        match self {
            Self::Unbounded => true,
            Self::Empty => false,
            Self::Rect(rect) => rect.contains(point),
        }
    }

    pub fn map<To: Copy>(self, transform: &Transform<Unit, To>) -> ClipRegion<To> {
        match self {
            Self::Unbounded => ClipRegion::Unbounded,
            Self::Empty => ClipRegion::Empty,
            Self::Rect(rect) => ClipRegion::Rect(transform.map_rect(rect)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClipStack<Unit> {
    regions: Vec<ClipRegion<Unit>>,
}

impl<Unit: Copy> ClipStack<Unit> {
    pub fn new() -> Self {
        Self {
            regions: vec![ClipRegion::Unbounded],
        }
    }

    pub fn current(&self) -> ClipRegion<Unit> {
        *self.regions.last().unwrap()
    }

    pub fn push(&mut self, region: ClipRegion<Unit>) -> ClipRegion<Unit> {
        let combined = self.current().intersect(region);
        self.regions.push(combined);
        combined
    }

    pub fn pop(&mut self) -> ClipRegion<Unit> {
        if self.regions.len() > 1 {
            self.regions.pop();
        }
        self.current()
    }

    pub fn depth(&self) -> usize {
        self.regions.len() - 1
    }
}

impl<Unit: Copy> Default for ClipStack<Unit> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct PhysicalPixelRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size<Unit> {
    pub width: f32,
    pub height: f32,
    unit: PhantomData<Unit>,
}

impl<Unit> Size<Unit> {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width: finite_non_negative(width),
            height: finite_non_negative(height),
            unit: PhantomData,
        }
    }

    pub fn is_empty(self) -> bool {
        self.width == 0.0 || self.height == 0.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect<Unit> {
    pub origin: Point<Unit>,
    pub size: Size<Unit>,
}

impl<Unit: Copy> Rect<Unit> {
    pub fn new(origin: Point<Unit>, size: Size<Unit>) -> Self {
        Self { origin, size }
    }

    pub fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::new(Point::new(x, y), Size::new(width, height))
    }

    pub fn min_x(self) -> f32 {
        self.origin.x
    }

    pub fn min_y(self) -> f32 {
        self.origin.y
    }

    pub fn max_x(self) -> f32 {
        self.origin.x + self.size.width
    }

    pub fn max_y(self) -> f32 {
        self.origin.y + self.size.height
    }

    pub fn contains(self, point: Point<Unit>) -> bool {
        point.x >= self.min_x()
            && point.x < self.max_x()
            && point.y >= self.min_y()
            && point.y < self.max_y()
    }

    pub fn intersection(self, other: Self) -> Option<Self> {
        let min_x = self.min_x().max(other.min_x());
        let min_y = self.min_y().max(other.min_y());
        let max_x = self.max_x().min(other.max_x());
        let max_y = self.max_y().min(other.max_y());
        (max_x > min_x && max_y > min_y)
            .then(|| Self::from_xywh(min_x, min_y, max_x - min_x, max_y - min_y))
    }
}

impl Point<Logical> {
    pub fn to_physical(self, scale_factor: ScaleFactor) -> Point<Physical> {
        scale_factor.logical_to_physical().map_point(self)
    }
}

impl Point<Physical> {
    pub fn to_logical(self, scale_factor: ScaleFactor) -> Point<Logical> {
        scale_factor.physical_to_logical().map_point(self)
    }
}

impl Size<Logical> {
    pub fn to_physical(self, scale_factor: ScaleFactor) -> Size<Physical> {
        Size::new(
            self.width * scale_factor.get(),
            self.height * scale_factor.get(),
        )
    }
}

impl Size<Physical> {
    pub fn to_logical(self, scale_factor: ScaleFactor) -> Size<Logical> {
        Size::new(
            self.width / scale_factor.get(),
            self.height / scale_factor.get(),
        )
    }
}

impl Rect<Logical> {
    pub fn to_physical(self, scale_factor: ScaleFactor) -> Rect<Physical> {
        Rect::new(
            self.origin.to_physical(scale_factor),
            self.size.to_physical(scale_factor),
        )
    }
}

impl Rect<Physical> {
    pub fn to_logical(self, scale_factor: ScaleFactor) -> Rect<Logical> {
        Rect::new(
            self.origin.to_logical(scale_factor),
            self.size.to_logical(scale_factor),
        )
    }

    pub fn snapped(self, policy: PixelSnap) -> Self {
        let (min_x, min_y, max_x, max_y) = match policy {
            PixelSnap::Nearest => (
                self.min_x().round(),
                self.min_y().round(),
                self.max_x().round(),
                self.max_y().round(),
            ),
            PixelSnap::Outward => (
                self.min_x().floor(),
                self.min_y().floor(),
                self.max_x().ceil(),
                self.max_y().ceil(),
            ),
            PixelSnap::Inward => (
                self.min_x().ceil(),
                self.min_y().ceil(),
                self.max_x().floor(),
                self.max_y().floor(),
            ),
        };
        Self::from_xywh(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    pub fn to_scissor(self, viewport: Size<Physical>) -> Option<PhysicalPixelRect> {
        let viewport = Self::new(Point::new(0.0, 0.0), viewport);
        let clipped = self.intersection(viewport)?.snapped(PixelSnap::Outward);
        let min_x = clipped.min_x().max(0.0).min(viewport.max_x());
        let min_y = clipped.min_y().max(0.0).min(viewport.max_y());
        let max_x = clipped.max_x().max(min_x).min(viewport.max_x());
        let max_y = clipped.max_y().max(min_y).min(viewport.max_y());
        let scissor = PhysicalPixelRect {
            x: min_x as u32,
            y: min_y as u32,
            width: (max_x - min_x) as u32,
            height: (max_y - min_y) as u32,
        };
        (scissor.width > 0 && scissor.height > 0).then_some(scissor)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Edges<Unit> {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
    unit: PhantomData<Unit>,
}

impl<Unit> Edges<Unit> {
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top: finite_non_negative(top),
            right: finite_non_negative(right),
            bottom: finite_non_negative(bottom),
            left: finite_non_negative(left),
            unit: PhantomData,
        }
    }

    pub fn all(value: f32) -> Self {
        Self::new(value, value, value, value)
    }

    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self::new(vertical, horizontal, vertical, horizontal)
    }

    pub fn horizontal(self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }
}

impl Edges<Logical> {
    pub fn to_physical(self, scale_factor: ScaleFactor) -> Edges<Physical> {
        Edges::new(
            self.top * scale_factor.get(),
            self.right * scale_factor.get(),
            self.bottom * scale_factor.get(),
            self.left * scale_factor.get(),
        )
    }
}

impl Edges<Physical> {
    pub fn to_logical(self, scale_factor: ScaleFactor) -> Edges<Logical> {
        Edges::new(
            self.top / scale_factor.get(),
            self.right / scale_factor.get(),
            self.bottom / scale_factor.get(),
            self.left / scale_factor.get(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Constraints<Unit> {
    pub min: Size<Unit>,
    pub max: Size<Unit>,
}

impl<Unit: Copy> Constraints<Unit> {
    pub fn new(min: Size<Unit>, max: Size<Unit>) -> Self {
        Self {
            min: Size::new(min.width.min(max.width), min.height.min(max.height)),
            max: Size::new(max.width.max(min.width), max.height.max(min.height)),
        }
    }

    pub fn tight(size: Size<Unit>) -> Self {
        Self::new(size, size)
    }

    pub fn loose(max: Size<Unit>) -> Self {
        Self::new(Size::new(0.0, 0.0), max)
    }

    pub fn unconstrained() -> Self {
        Self {
            min: Size::new(0.0, 0.0),
            max: Size {
                width: f32::INFINITY,
                height: f32::INFINITY,
                unit: PhantomData,
            },
        }
    }

    pub fn constrain(self, size: Size<Unit>) -> Size<Unit> {
        Size::new(
            size.width.clamp(self.min.width, self.max.width),
            size.height.clamp(self.min.height, self.max.height),
        )
    }
}

impl<Unit: Copy> Default for Constraints<Unit> {
    fn default() -> Self {
        Self::unconstrained()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform<From, To> {
    pub matrix: [f32; 6],
    units: PhantomData<(From, To)>,
}

impl<Unit> Transform<Unit, Unit> {
    pub const fn identity() -> Self {
        Self::from_matrix([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
    }

    pub const fn translation(x: f32, y: f32) -> Self {
        Self::from_matrix([1.0, 0.0, 0.0, 1.0, x, y])
    }

    pub const fn scale(x: f32, y: f32) -> Self {
        Self::from_matrix([x, 0.0, 0.0, y, 0.0, 0.0])
    }
}

impl<From, To> Transform<From, To> {
    pub const fn from_matrix(matrix: [f32; 6]) -> Self {
        Self {
            matrix,
            units: PhantomData,
        }
    }

    pub fn map_point(&self, point: Point<From>) -> Point<To> {
        let [m11, m12, m21, m22, tx, ty] = self.matrix;
        Point::new(
            m11 * point.x + m21 * point.y + tx,
            m12 * point.x + m22 * point.y + ty,
        )
    }

    pub fn map_rect(&self, rect: Rect<From>) -> Rect<To>
    where
        From: Copy,
        To: Copy,
    {
        let corners = [
            self.map_point(Point::new(rect.min_x(), rect.min_y())),
            self.map_point(Point::new(rect.max_x(), rect.min_y())),
            self.map_point(Point::new(rect.min_x(), rect.max_y())),
            self.map_point(Point::new(rect.max_x(), rect.max_y())),
        ];
        let min_x = corners
            .iter()
            .map(|point| point.x)
            .reduce(f32::min)
            .unwrap();
        let min_y = corners
            .iter()
            .map(|point| point.y)
            .reduce(f32::min)
            .unwrap();
        let max_x = corners
            .iter()
            .map(|point| point.x)
            .reduce(f32::max)
            .unwrap();
        let max_y = corners
            .iter()
            .map(|point| point.y)
            .reduce(f32::max)
            .unwrap();
        Rect::from_xywh(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    pub fn then<Next>(&self, next: &Transform<To, Next>) -> Transform<From, Next> {
        let [a11, a12, a21, a22, atx, aty] = self.matrix;
        let [b11, b12, b21, b22, btx, bty] = next.matrix;
        Transform::from_matrix([
            b11 * a11 + b21 * a12,
            b12 * a11 + b22 * a12,
            b11 * a21 + b21 * a22,
            b12 * a21 + b22 * a22,
            b11 * atx + b21 * aty + btx,
            b12 * atx + b22 * aty + bty,
        ])
    }
}

pub type LogicalPoint = Point<Logical>;
pub type LogicalSize = Size<Logical>;
pub type LogicalRect = Rect<Logical>;
pub type LogicalEdges = Edges<Logical>;
pub type LogicalConstraints = Constraints<Logical>;
pub type LogicalTransform = Transform<Logical, Logical>;
pub type PhysicalPoint = Point<Physical>;
pub type PhysicalSize = Size<Physical>;
pub type PhysicalRect = Rect<Physical>;
pub type PhysicalEdges = Edges<Physical>;
pub type PhysicalTransform = Transform<Physical, Physical>;

fn finite_non_negative(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else if value.is_finite() {
        value.max(0.0)
    } else if value.is_sign_positive() {
        f32::MAX
    } else {
        0.0
    }
}

fn finite_signed(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::{
        ClipRegion, ClipStack, LogicalConstraints, LogicalEdges, LogicalPoint, LogicalRect,
        LogicalSize, LogicalTransform, Overflow, PhysicalPixelRect, PhysicalRect, PhysicalSize,
        PixelSnap, ScaleFactor,
    };

    #[test]
    fn sizes_and_edges_reject_invalid_negative_geometry() {
        assert_eq!(LogicalSize::new(-2.0, f32::NAN), LogicalSize::new(0.0, 0.0));
        assert_eq!(LogicalSize::new(f32::INFINITY, 2.0).width, f32::MAX);
        assert_eq!(
            LogicalEdges::new(-1.0, 2.0, 3.0, f32::NAN),
            LogicalEdges::new(0.0, 2.0, 3.0, 0.0)
        );
        assert_eq!(LogicalEdges::symmetric(2.0, 3.0).horizontal(), 6.0);
        assert_eq!(LogicalEdges::symmetric(2.0, 3.0).vertical(), 4.0);
    }

    #[test]
    fn rectangles_use_half_open_bounds_and_strict_intersections() {
        let rect = LogicalRect::from_xywh(10.0, 20.0, 30.0, 40.0);

        assert!(rect.contains(LogicalPoint::new(10.0, 20.0)));
        assert!(rect.contains(LogicalPoint::new(39.999, 59.999)));
        assert!(!rect.contains(LogicalPoint::new(40.0, 60.0)));
        assert_eq!(
            rect.intersection(LogicalRect::from_xywh(30.0, 40.0, 20.0, 30.0)),
            Some(LogicalRect::from_xywh(30.0, 40.0, 10.0, 20.0))
        );
        assert_eq!(
            rect.intersection(LogicalRect::from_xywh(40.0, 20.0, 10.0, 10.0)),
            None
        );
    }

    #[test]
    fn constraints_normalize_and_clamp_each_axis() {
        let constraints =
            LogicalConstraints::new(LogicalSize::new(100.0, 20.0), LogicalSize::new(40.0, 80.0));

        assert_eq!(constraints.min, LogicalSize::new(40.0, 20.0));
        assert_eq!(constraints.max, LogicalSize::new(100.0, 80.0));
        assert_eq!(
            constraints.constrain(LogicalSize::new(10.0, 100.0)),
            LogicalSize::new(40.0, 80.0)
        );
        assert_eq!(
            LogicalConstraints::tight(LogicalSize::new(12.0, 14.0))
                .constrain(LogicalSize::new(99.0, 1.0)),
            LogicalSize::new(12.0, 14.0)
        );
    }

    #[test]
    fn transforms_compose_in_application_order() {
        let translate = LogicalTransform::translation(10.0, 20.0);
        let scale = LogicalTransform::scale(2.0, 3.0);
        let combined = translate.then(&scale);

        assert_eq!(
            combined.map_point(LogicalPoint::new(4.0, 5.0)),
            LogicalPoint::new(28.0, 75.0)
        );
        assert_eq!(
            LogicalTransform::identity().map_point(LogicalPoint::new(4.0, 5.0)),
            LogicalPoint::new(4.0, 5.0)
        );
    }

    #[test]
    fn transformed_rect_is_axis_aligned_bounding_box() {
        let rotate_quarter = LogicalTransform::from_matrix([0.0, 1.0, -1.0, 0.0, 0.0, 0.0]);
        let rect = LogicalRect::from_xywh(10.0, 20.0, 30.0, 40.0);

        assert_eq!(
            rotate_quarter.map_rect(rect),
            LogicalRect::from_xywh(-60.0, 10.0, 40.0, 30.0)
        );
    }

    #[test]
    fn scale_factor_rejects_invalid_values() {
        assert_eq!(ScaleFactor::new(1.25).unwrap().get(), 1.25);
        assert_eq!(ScaleFactor::new(0.0), None);
        assert_eq!(ScaleFactor::new(-1.0), None);
        assert_eq!(ScaleFactor::new(f32::NAN), None);
        assert_eq!(
            ScaleFactor::new_or_one(f32::INFINITY),
            ScaleFactor::default()
        );
    }

    #[test]
    fn logical_physical_mappings_preserve_fractional_geometry() {
        let scale = ScaleFactor::new(1.25).unwrap();
        let point = LogicalPoint::new(-2.5, 3.25);
        let size = LogicalSize::new(10.5, 20.25);
        let rect = LogicalRect::new(point, size);
        let edges = LogicalEdges::new(1.0, 2.0, 3.0, 4.0);

        assert_eq!(point.to_physical(scale).to_logical(scale), point);
        assert_eq!(size.to_physical(scale).to_logical(scale), size);
        assert_eq!(rect.to_physical(scale).to_logical(scale), rect);
        assert_eq!(edges.to_physical(scale).to_logical(scale), edges);
        assert_eq!(
            scale.logical_to_physical().map_point(point),
            point.to_physical(scale)
        );
    }

    #[test]
    fn pixel_snapping_uses_explicit_boundary_rules() {
        let rect = PhysicalRect::from_xywh(-1.25, 2.25, 3.5, 4.5);

        assert_eq!(
            rect.snapped(PixelSnap::Outward),
            PhysicalRect::from_xywh(-2.0, 2.0, 5.0, 5.0)
        );
        assert_eq!(
            rect.snapped(PixelSnap::Nearest),
            PhysicalRect::from_xywh(-1.0, 2.0, 3.0, 5.0)
        );
        assert_eq!(
            rect.snapped(PixelSnap::Inward),
            PhysicalRect::from_xywh(-1.0, 3.0, 3.0, 3.0)
        );
    }

    #[test]
    fn point_coordinates_reject_non_finite_values() {
        assert_eq!(
            LogicalPoint::new(f32::NAN, f32::INFINITY),
            LogicalPoint::new(0.0, 0.0)
        );
        assert_eq!(
            LogicalPoint::new(-12.0, -4.0),
            LogicalPoint::new(-12.0, -4.0)
        );
    }

    #[test]
    fn overflow_creates_unbounded_or_bounded_regions() {
        let bounds = LogicalRect::from_xywh(10.0, 20.0, 30.0, 40.0);

        assert_eq!(
            ClipRegion::from_overflow(bounds, Overflow::Visible),
            ClipRegion::Unbounded
        );
        assert_eq!(
            ClipRegion::from_overflow(bounds, Overflow::Clip),
            ClipRegion::Rect(bounds)
        );
        assert_eq!(
            ClipRegion::from_overflow(
                LogicalRect::from_xywh(10.0, 20.0, 0.0, 40.0),
                Overflow::Clip
            ),
            ClipRegion::Empty
        );
    }

    #[test]
    fn nested_clip_stack_intersects_and_restores_regions() {
        let outer = LogicalRect::from_xywh(0.0, 0.0, 100.0, 100.0);
        let inner = LogicalRect::from_xywh(80.0, 20.0, 50.0, 50.0);
        let mut clips = ClipStack::new();

        assert_eq!(clips.push(ClipRegion::Rect(outer)), ClipRegion::Rect(outer));
        assert_eq!(
            clips.push(ClipRegion::Rect(inner)),
            ClipRegion::Rect(LogicalRect::from_xywh(80.0, 20.0, 20.0, 50.0))
        );
        assert_eq!(clips.depth(), 2);
        assert!(!clips.current().contains(LogicalPoint::new(79.0, 30.0)));
        assert!(clips.current().contains(LogicalPoint::new(90.0, 30.0)));
        assert_eq!(clips.pop(), ClipRegion::Rect(outer));
        assert_eq!(clips.pop(), ClipRegion::Unbounded);
        assert_eq!(clips.pop(), ClipRegion::Unbounded);
    }

    #[test]
    fn clip_regions_follow_transforms() {
        let clip = ClipRegion::Rect(LogicalRect::from_xywh(5.0, 10.0, 20.0, 30.0));
        let transform =
            LogicalTransform::translation(10.0, -5.0).then(&LogicalTransform::scale(2.0, 2.0));

        assert_eq!(
            clip.map(&transform),
            ClipRegion::Rect(LogicalRect::from_xywh(30.0, 10.0, 40.0, 60.0))
        );
        assert_eq!(
            ClipRegion::<super::Logical>::Empty.map(&transform),
            ClipRegion::Empty
        );
    }

    #[test]
    fn physical_scissors_round_outward_and_clamp_to_viewport() {
        let viewport = PhysicalSize::new(100.0, 80.0);

        assert_eq!(
            PhysicalRect::from_xywh(-2.2, 10.2, 20.4, 70.5).to_scissor(viewport),
            Some(PhysicalPixelRect {
                x: 0,
                y: 10,
                width: 19,
                height: 70,
            })
        );
        assert_eq!(
            PhysicalRect::from_xywh(100.0, 0.0, 20.0, 20.0).to_scissor(viewport),
            None
        );
    }
}
