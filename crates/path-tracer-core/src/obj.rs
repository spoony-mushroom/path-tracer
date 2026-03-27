use std::fs;
use std::io;

use crate::hittable::Shape;
use crate::material::Material;
use crate::vec3::Point3;

/// Load triangles from a Wavefront OBJ file.
///
/// Only vertex positions (`v`) and faces (`f`) are parsed. Normals and texture
/// coordinates are ignored. Polygons with more than 3 vertices are
/// fan-triangulated from the first vertex.
pub fn load_obj(path: &str, material: Material) -> io::Result<Vec<Shape>> {
    let contents = fs::read_to_string(path)?;
    Ok(parse_obj(&contents, material))
}

/// Parse OBJ text into a list of triangle shapes.
pub fn parse_obj(source: &str, material: Material) -> Vec<Shape> {
    let mut vertices: Vec<Point3> = Vec::new();
    let mut shapes: Vec<Shape> = Vec::new();

    for line in source.lines() {
        let line = line.trim();
        if line.starts_with("v ") {
            if let Some(v) = parse_vertex(line) {
                vertices.push(v);
            }
        } else if line.starts_with("f ") {
            let indices = parse_face(line);
            // Fan triangulation: (0, 1, 2), (0, 2, 3), (0, 3, 4), ...
            for i in 1..indices.len().saturating_sub(1) {
                let i0 = indices[0];
                let i1 = indices[i];
                let i2 = indices[i + 1];
                if let (Some(&v0), Some(&v1), Some(&v2)) =
                    (vertices.get(i0), vertices.get(i1), vertices.get(i2))
                {
                    shapes.push(Shape::triangle(v0, v1, v2, material));
                }
            }
        }
    }

    shapes
}

fn parse_vertex(line: &str) -> Option<Point3> {
    let mut parts = line.split_whitespace().skip(1);
    let x: f64 = parts.next()?.parse().ok()?;
    let y: f64 = parts.next()?.parse().ok()?;
    let z: f64 = parts.next()?.parse().ok()?;
    Some(Point3::new(x, y, z))
}

/// Parse a face line, handling formats like `f 1 2 3`, `f 1/2/3 4/5/6`,
/// and `f 1//3 4//6`. Returns 0-based vertex indices.
fn parse_face(line: &str) -> Vec<usize> {
    line.split_whitespace()
        .skip(1)
        .filter_map(|token| {
            let idx_str = token.split('/').next()?;
            let idx: usize = idx_str.parse().ok()?;
            Some(idx - 1) // OBJ indices are 1-based
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec3::Color;

    fn test_mat() -> Material {
        Material::lambertian(Color::new(0.5, 0.5, 0.5))
    }

    #[test]
    fn parse_single_triangle() {
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";
        let shapes = parse_obj(obj, test_mat());
        assert_eq!(shapes.len(), 1);
    }

    #[test]
    fn parse_quad_fan_triangulated() {
        let obj = "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n";
        let shapes = parse_obj(obj, test_mat());
        assert_eq!(shapes.len(), 2);
    }

    #[test]
    fn parse_face_with_slashes() {
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1//1 2//2 3//3\n";
        let shapes = parse_obj(obj, test_mat());
        assert_eq!(shapes.len(), 1);
    }

    #[test]
    fn ignores_comments_and_other_lines() {
        let obj = "# comment\no cube\nv 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";
        let shapes = parse_obj(obj, test_mat());
        assert_eq!(shapes.len(), 1);
    }
}
