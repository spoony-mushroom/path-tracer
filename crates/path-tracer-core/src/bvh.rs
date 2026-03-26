use crate::aabb::Aabb;
use crate::hittable::{HitRecord, Hittable, Interval, Shape};
use crate::ray::Ray;

/// A flat BVH node stored contiguously in memory.
///
/// For interior nodes, the left child is always at `self_index + 1` (implicit),
/// and `right_child` stores the index of the right child.
/// For leaf nodes, `shape_index` indexes into the `Bvh::shapes` array.
struct FlatNode {
    bbox: Aabb,
    /// Interior: right child index. Leaf: shape index.
    offset: u32,
    is_leaf: bool,
}

/// A Bounding Volume Hierarchy stored as a flat array for cache-friendly traversal.
pub struct Bvh {
    nodes: Vec<FlatNode>,
    shapes: Vec<Shape>,
}

/// Temporary recursive tree used during construction.
enum BuildNode {
    Leaf {
        bbox: Aabb,
        shape_index: usize,
    },
    Interior {
        bbox: Aabb,
        left: Box<BuildNode>,
        right: Box<BuildNode>,
    },
}

impl Bvh {
    /// Build a BVH from a list of shapes using median split along the longest axis.
    pub fn build(shapes: Vec<Shape>) -> Self {
        assert!(!shapes.is_empty(), "BVH requires at least one shape");

        let indices: Vec<usize> = (0..shapes.len()).collect();
        let tree = build_recursive(&shapes, indices);

        let mut nodes = Vec::new();
        flatten(&tree, &mut nodes);

        Self { nodes, shapes }
    }
}

fn build_recursive(shapes: &[Shape], mut indices: Vec<usize>) -> BuildNode {
    if indices.len() == 1 {
        let i = indices[0];
        return BuildNode::Leaf {
            bbox: shapes[i].bounding_box(),
            shape_index: i,
        };
    }

    let overall_bbox = indices
        .iter()
        .map(|&i| shapes[i].bounding_box())
        .reduce(Aabb::surrounding)
        .unwrap();

    let axis = overall_bbox.longest_axis();

    indices.sort_by(|&a, &b| {
        let ca = shapes[a].bounding_box().centroid().axis(axis);
        let cb = shapes[b].bounding_box().centroid().axis(axis);
        ca.partial_cmp(&cb).unwrap()
    });

    let mid = indices.len() / 2;
    let right_indices = indices.split_off(mid);

    let left = Box::new(build_recursive(shapes, indices));
    let right = Box::new(build_recursive(shapes, right_indices));

    BuildNode::Interior {
        bbox: overall_bbox,
        left,
        right,
    }
}

/// Flatten the recursive tree into depth-first order.
/// Left child is always at `parent + 1`. Right child index is backpatched.
fn flatten(node: &BuildNode, nodes: &mut Vec<FlatNode>) {
    match node {
        BuildNode::Leaf {
            bbox,
            shape_index,
        } => {
            nodes.push(FlatNode {
                bbox: *bbox,
                offset: *shape_index as u32,
                is_leaf: true,
            });
        }
        BuildNode::Interior { bbox, left, right } => {
            let self_index = nodes.len();
            // Push a placeholder; we'll backpatch the right child offset.
            nodes.push(FlatNode {
                bbox: *bbox,
                offset: 0,
                is_leaf: false,
            });

            // Left child is next (self_index + 1).
            flatten(left, nodes);

            // Right child starts here.
            let right_index = nodes.len();
            nodes[self_index].offset = right_index as u32;
            flatten(right, nodes);
        }
    }
}

impl Hittable for Bvh {
    fn hit(&self, ray: &Ray, t_range: Interval) -> Option<HitRecord> {
        let mut stack = [0u32; 64];
        let mut stack_ptr: usize = 0;
        stack[0] = 0;

        let mut closest = t_range.max;
        let mut result = None;

        loop {
            let node_idx = stack[stack_ptr] as usize;
            let node = &self.nodes[node_idx];

            if node.bbox.hit(ray, Interval::new(t_range.min, closest)) {
                if node.is_leaf {
                    let shape = &self.shapes[node.offset as usize];
                    if let Some(hit) = shape.hit(ray, Interval::new(t_range.min, closest)) {
                        closest = hit.t;
                        result = Some(hit);
                    }
                } else {
                    // Push right child, then left (left will be processed first).
                    stack[stack_ptr] = node.offset; // right child
                    stack_ptr += 1;
                    stack[stack_ptr] = (node_idx + 1) as u32; // left child
                    continue; // don't decrement stack_ptr
                }
            }

            if stack_ptr == 0 {
                break;
            }
            stack_ptr -= 1;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Material;
    use crate::vec3::{Color, Point3, Vec3};

    fn test_sphere(x: f64, z: f64) -> Shape {
        Shape::sphere(
            Point3::new(x, 0.0, z),
            0.5,
            Material::lambertian(Color::new(0.5, 0.5, 0.5)),
        )
    }

    #[test]
    fn test_build_single() {
        let bvh = Bvh::build(vec![test_sphere(0.0, 0.0)]);
        assert_eq!(bvh.nodes.len(), 1);
        assert!(bvh.nodes[0].is_leaf);
    }

    #[test]
    fn test_build_multiple() {
        let shapes = vec![
            test_sphere(-2.0, 0.0),
            test_sphere(0.0, 0.0),
            test_sphere(2.0, 0.0),
        ];
        let bvh = Bvh::build(shapes);
        assert!(!bvh.nodes[0].is_leaf);
        assert!(bvh.nodes.len() > 1);
    }

    #[test]
    fn test_hit_finds_closest() {
        let shapes = vec![
            test_sphere(0.0, -3.0), // farther
            test_sphere(0.0, -1.0), // closer
        ];
        let bvh = Bvh::build(shapes);

        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = bvh.hit(&ray, Interval::new(0.001, f64::INFINITY));
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert!((hit.point.z - -0.5).abs() < 0.1);
    }

    #[test]
    fn test_miss() {
        let shapes = vec![test_sphere(0.0, 0.0)];
        let bvh = Bvh::build(shapes);

        let ray = Ray::new(Point3::new(10.0, 10.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
        let hit = bvh.hit(&ray, Interval::new(0.001, f64::INFINITY));
        assert!(hit.is_none());
    }

    #[test]
    fn test_flat_layout_left_child_is_next() {
        let shapes = vec![
            test_sphere(-2.0, 0.0),
            test_sphere(2.0, 0.0),
        ];
        let bvh = Bvh::build(shapes);
        // Root is interior at index 0, left child at index 1.
        assert!(!bvh.nodes[0].is_leaf);
        assert!(bvh.nodes[1].is_leaf);
        // Right child is at the stored offset.
        let right_idx = bvh.nodes[0].offset as usize;
        assert!(bvh.nodes[right_idx].is_leaf);
    }
}
