use crate::aabb::Aabb;
use crate::hittable::{HitRecord, Hittable, Interval, Shape};
use crate::ray::Ray;

/// A Bounding Volume Hierarchy node.
///
/// Organizes shapes into a binary tree of axis-aligned bounding boxes,
/// reducing ray intersection tests from O(n) to O(log n).
pub enum BvhNode {
    Leaf {
        bbox: Aabb,
        shape: Shape,
    },
    Interior {
        bbox: Aabb,
        left: Box<BvhNode>,
        right: Box<BvhNode>,
    },
}

impl BvhNode {
    /// Build a BVH from a list of shapes using median split along the longest axis.
    pub fn build(mut shapes: Vec<Shape>) -> Self {
        assert!(!shapes.is_empty(), "BVH requires at least one shape");

        if shapes.len() == 1 {
            let shape = shapes.pop().unwrap();
            let bbox = shape.bounding_box();
            return Self::Leaf { bbox, shape };
        }

        let overall_bbox = shapes
            .iter()
            .map(|s| s.bounding_box())
            .reduce(Aabb::surrounding)
            .unwrap();

        let axis = overall_bbox.longest_axis();

        shapes.sort_by(|a, b| {
            let ca = a.bounding_box().centroid().axis(axis);
            let cb = b.bounding_box().centroid().axis(axis);
            ca.partial_cmp(&cb).unwrap()
        });

        let mid = shapes.len() / 2;
        let right_shapes = shapes.split_off(mid);

        let left = Box::new(Self::build(shapes));
        let right = Box::new(Self::build(right_shapes));

        Self::Interior {
            bbox: overall_bbox,
            left,
            right,
        }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: &Ray, t_range: Interval) -> Option<HitRecord> {
        match self {
            Self::Leaf { bbox, shape } => {
                if bbox.hit(ray, t_range) {
                    shape.hit(ray, t_range)
                } else {
                    None
                }
            }
            Self::Interior { bbox, left, right } => {
                if !bbox.hit(ray, t_range) {
                    return None;
                }
                let hit_left = left.hit(ray, t_range);
                let right_range = Interval::new(
                    t_range.min,
                    hit_left.as_ref().map_or(t_range.max, |h| h.t),
                );
                let hit_right = right.hit(ray, right_range);
                hit_right.or(hit_left)
            }
        }
    }
}
