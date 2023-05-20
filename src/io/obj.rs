use std::{
    fs::File,
    io::{BufRead, BufReader},
    sync::Arc,
};

use crate::{
    base::{constants::Float, transform::Transform},
    geometries::{normal::Normal, point2::Point2F, point3::Point3},
    shapes::triangle::{Triangle, TriangleMesh},
};

pub struct OBJ;

impl OBJ {
    pub fn read(path: &str, transform: Transform) -> Vec<Triangle> {
        let file = File::open(path).expect("Failed to open OBJ file");
        let reader = BufReader::new(file);

        let mut vertices = vec![];
        let mut vertex_indices = vec![];

        let mut normals = vec![];
        let mut normal_indices = vec![];

        let mut uvs = vec![];
        let mut uv_indices = vec![];

        for line in reader.lines() {
            let line = line.as_ref().expect("Failed to read line").trim();

            if line.starts_with("f") {
                let mut values = line.split_whitespace();
                values.next();

                for v in values {
                    if v.contains("//") {
                        let indices = v.split("//").collect::<Vec<&str>>();
                        debug_assert!(indices.len() == 2);

                        vertex_indices.push(indices[0].parse::<usize>().unwrap());
                        normal_indices.push(indices[1].parse::<usize>().unwrap());
                    } else if v.contains("/") {
                        let indices = v.split("/").collect::<Vec<&str>>();
                        debug_assert!(indices.len() == 2 || indices.len() == 3);

                        vertex_indices.push(indices[0].parse::<usize>().unwrap());
                        uv_indices.push(indices[1].parse::<usize>().unwrap());
                        if indices.len() == 3 {
                            normal_indices.push(indices[2].parse::<usize>().unwrap());
                        }
                    } else {
                        vertex_indices.push(v.parse::<usize>().unwrap());
                    }
                }
            }

            if line.starts_with("v") {
                let mut values = line.split_whitespace();
                values.next();

                let p: Vec<Float> = values.map(|v| v.parse().unwrap()).collect();

                if line.starts_with("vt") {
                    debug_assert!(p.len() == 2);
                    uvs.push(Point2F::new(p[0], p[1]));
                } else if line.starts_with("vn") {
                    debug_assert!(p.len() == 3);
                    normals.push(Normal::new(p[0], p[1], p[2]));
                } else {
                    debug_assert!(p.len() == 3);
                    vertices.push(Point3::new(p[0], p[1], p[2]));
                }
            }
        }

        let mut position = Vec::with_capacity(vertex_indices.len());
        let mut normal = Vec::with_capacity(vertex_indices.len());
        let mut uv = Vec::with_capacity(vertex_indices.len());
        for i in 0..vertex_indices.len() {
            let vertex_index = vertex_indices[i];
            position.push(vertices[vertex_index - 1]);

            if let Some(normal_index) = normal_indices.get(i) {
                normal.push(normals[normal_index - 1]);
            }

            if let Some(uv_index) = uv_indices.get(i) {
                uv.push(uvs[uv_index - 1]);
            }
        }

        let normal = if normal.is_empty() {
            None
        } else {
            Some(normal)
        };
        let uv = if uv.is_empty() { None } else { Some(uv) };

        let mesh = Arc::new(TriangleMesh::new(crate::TriangleMeshOptions {
            object_to_world: transform.clone(),
            position,
            tangent: None,
            normal,
            uv,
        }));

        let transform = Arc::new(transform.inverse());

        let num_triangles = vertex_indices.len() / 3;
        let mut triangles = Vec::with_capacity(num_triangles);

        for index in 0..num_triangles {
            triangles.push(Triangle::new(crate::TriangleOptions {
                world_to_object: transform.clone(),
                reverse_orientation: false,
                mesh: mesh.clone(),
                index,
            }));
        }

        triangles
    }
}
