use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    ptr,
    sync::Arc,
};

use crate::{
    base::shape::Shape,
    geometries::{normal::Normal, point3::Point3, vec3::Vec3},
    transform::Transform,
    utils::math::{Float, PI},
};

#[derive(Debug, Clone, PartialEq)]
struct SDVertex {
    position: Point3,
    start_face: Option<SDFace>,
    child: Option<Box<SDVertex>>,
    regular: bool,
    boundary: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct SDFace {
    vertices: Vec<SDVertex>,
    faces: Vec<SDFace>,
    children: Vec<SDFace>,
}

#[derive(Debug)]
struct SDEdge {
    vertices: Vec<SDVertex>,
    faces: Vec<SDFace>,
    f0_edge_num: i32,
}

impl SDVertex {
    pub fn new(position: Point3) -> Self {
        Self {
            position,
            start_face: None,
            child: None,
            regular: false,
            boundary: false,
        }
    }

    pub fn valence(&self) -> i32 {
        let start_face = self.start_face.as_ref().unwrap();
        let mut face = self.start_face.as_ref().unwrap();
        let mut num_faces = 1;

        if !self.boundary {
            // Compute valence of interior vertex.
            loop {
                if let Some(f) = face.next_face(&self) {
                    if !ptr::eq(start_face, f) {
                        face = f;
                        num_faces += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            num_faces
        } else {
            // Compute valence of boundary vertex.
            loop {
                if let Some(f) = face.next_face(&self) {
                    face = f;
                    num_faces += 1;
                } else {
                    break;
                }
            }
            face = start_face;
            loop {
                if let Some(f) = face.previous_face(&self) {
                    face = f;
                    num_faces += 1;
                } else {
                    break;
                }
            }

            num_faces + 1
        }
    }

    pub fn one_ring(&self, points: &mut Vec<Point3>) {
        let start_face = self.start_face.as_ref().unwrap();
        let mut face = self.start_face.as_ref().unwrap();

        let mut point_index: usize = 1;
        if !self.boundary {
            // Get one-ring vertices for interior vertex.
            loop {
                if let Some(f) = face.next_face(&self) {
                    if !ptr::eq(start_face, f) {
                        face = f;
                        points[point_index].clone_from(&face.next_vertex(&self).unwrap().position);
                        point_index += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        } else {
            // Get one-ring vertices for boundary vertex.
            loop {
                if let Some(f) = face.next_face(&self) {
                    face = f;
                } else {
                    break;
                }
            }
            points[point_index].clone_from(&face.next_vertex(&self).unwrap().position);
            point_index += 1;

            face = start_face;
            loop {
                if let Some(f) = face.previous_face(&self) {
                    face = f;
                    points[point_index].clone_from(&face.previous_vertex(&self).unwrap().position);
                    point_index += 1;
                } else {
                    break;
                }
            }
        }
    }

    pub fn weight_one_ring(&self, beta: Float) -> Point3 {
        let valence = self.valence();

        let mut point_ring = vec![Point3::default(); valence as usize];
        self.one_ring(&mut point_ring);

        let mut point = (1.0 - (valence as Float) * beta) * self.position;
        for p in point_ring {
            point += beta * p
        }

        point
    }

    pub fn weight_boundary(&self, beta: Float) -> Point3 {
        let valence = self.valence();

        let mut point_ring = vec![Point3::default(); valence as usize];
        self.one_ring(&mut point_ring);

        let mut point = (1.0 - 2.0 * beta) * self.position;
        point += beta * point_ring[0];
        point += beta * point_ring[(valence - 1) as usize];

        point
    }
}

impl SDFace {
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(3),
            faces: Vec::with_capacity(3),
            children: Vec::with_capacity(4),
        }
    }

    pub fn get_vertex_index(&self, vertex: &SDVertex) -> usize {
        for (i, v) in self.vertices.iter().enumerate() {
            if ptr::eq(vertex, v) {
                return i;
            }
        }
        panic!("Logic error in SDFace::vertex_index");
    }

    pub fn next_face(&self, vertex: &SDVertex) -> Option<&SDFace> {
        self.faces.get(self.get_vertex_index(vertex))
    }

    pub fn previous_face(&self, vertex: &SDVertex) -> Option<&SDFace> {
        self.faces.get(previous(self.get_vertex_index(vertex)))
    }

    pub fn next_vertex(&self, vertex: &SDVertex) -> Option<&SDVertex> {
        self.vertices.get(next(self.get_vertex_index(vertex)))
    }

    pub fn previous_vertex(&self, vertex: &SDVertex) -> Option<&SDVertex> {
        self.vertices.get(previous(self.get_vertex_index(vertex)))
    }

    pub fn other_vertex(&self, v0: &SDVertex, v1: &SDVertex) -> &SDVertex {
        for (i, v) in self.vertices.iter().enumerate() {
            if !ptr::eq(v0, v) && !ptr::eq(v1, v) {
                return v;
            }
        }
        panic!("Logic error in SDFace::other_vertex");
    }
}

impl SDEdge {
    pub fn new(v0: SDVertex, v1: SDVertex) -> Self {
        // Sort vertices in struct by their address. This makes
        // sure different edges with the same vertices will
        // produce the same ording.
        let v0_addr = &v0 as *const SDVertex as usize;
        let v1_addr = &v0 as *const SDVertex as usize;
        let vertices = if v0_addr < v1_addr {
            vec![v0, v1]
        } else {
            vec![v1, v0]
        };

        Self {
            vertices,
            faces: Vec::with_capacity(2),
            f0_edge_num: -1,
        }
    }
}

impl PartialEq for SDEdge {
    fn eq(&self, other: &Self) -> bool {
        self.vertices[0].position == self.vertices[0].position
            && other.vertices[1].position == other.vertices[1].position
    }
}

impl Eq for SDEdge {}

impl Hash for SDEdge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let p0_addr = &self.vertices[0].position as *const Point3 as usize;
        let p1_addr = &self.vertices[1].position as *const Point3 as usize;
        p0_addr.hash(state);
        p1_addr.hash(state);
    }
}

fn previous(i: usize) -> usize {
    (i + 2) % 3
}

fn next(i: usize) -> usize {
    (i + 1) % 3
}

fn beta(valence: i32) -> Float {
    if valence == 3 {
        3.0 / 16.0
    } else {
        3.0 / (8.0 * valence as Float)
    }
}

fn loop_gamma(valence: i32) -> Float {
    1.0 / ((valence as Float) + 3.0 / (8.0 * beta(valence)))
}

pub fn loop_subdivision<'a>(
    object_to_world: &Transform,
    world_to_object: &Transform,
    reverse_orientation: bool,
    num_levels: usize,
    num_indices: usize,
    num_vertices: usize,
    vertex_indices: Vec<usize>,
    positions: Vec<Point3>,
) -> Vec<Arc<dyn Shape<'a>>> {
    let num_faces = num_indices / 3;

    let mut vertices: Vec<SDVertex> = Vec::with_capacity(num_vertices);
    let mut faces: Vec<SDFace> = Vec::with_capacity(num_faces);

    // Allocate vertices and faces for subdivision.
    for i in 0..num_vertices {
        vertices.push(SDVertex::new(positions[i]));
    }
    for i in 0..num_faces {
        faces.push(SDFace::new());
    }

    // Set face to vertex.
    let mut vertex_index: usize = 0;
    for (i, face) in faces.iter_mut().enumerate() {
        for j in 0..3 {
            let vertex = vertices.get_mut(vertex_index + j).unwrap();
            face.vertices[j] = vertex.clone();
            vertex.start_face = Some(face.clone());
        }
        vertex_index += 3;
    }

    // Set neighbors in faces.
    let mut edges: Vec<SDEdge> = vec![];
    for face in faces.iter() {
        for edge_num in 0..3 {
            // Update neighbor for edge_num.
            let v0 = edge_num;
            let v1 = next(edge_num);
            let mut edge = SDEdge::new(face.vertices[v0].clone(), face.vertices[v1].clone());
            if let Some(edge_index) = edges.iter().position(|e| e == &edge) {
                // Handle previously seen edge.
                let edge = edges.get_mut(edge_index).unwrap();
                edge.faces[0].faces[edge.f0_edge_num as usize] = face.clone();
                edges.remove(edge_index);
            } else {
                // Handle new edge.
                edge.faces[0] = face.clone();
                edge.f0_edge_num = edge_num as i32;
                edges.push(edge);
            }
        }
    }

    // Finish vertex initialization.
    for vertex in vertices.iter_mut() {
        let mut face = vertex.start_face.as_ref();
        loop {
            if let Some(f) = face {
                face = f.next_face(&vertex);
            } else {
                break;
            }
        }
        vertex.boundary = face.is_none();
        if !vertex.boundary && vertex.valence() == 6 {
            vertex.regular = true;
        } else if vertex.boundary && vertex.valence() == 4 {
            vertex.regular = true;
        } else {
            vertex.regular = false;
        }
    }

    // Refine into triangles.
    for i in 0..num_levels {
        let mut new_vertices: Vec<SDVertex> = Vec::with_capacity(vertices.len());
        let mut new_faces: Vec<SDFace> = Vec::with_capacity(faces.len());

        // Allocate next level of children in mesh tree.
        for vertex in vertices.iter_mut() {
            let mut child_vertex = SDVertex::new(Point3::default());
            child_vertex.regular = vertex.regular;
            child_vertex.boundary = vertex.boundary;
            vertex.child = Some(Box::new(child_vertex.clone()));
            new_vertices.push(child_vertex);
        }
        for face in faces.iter_mut() {
            for k in 0..4 {
                face.children[k] = SDFace::new();
                new_faces.push(face.children[k].clone());
            }
        }

        // Update vertex positions for even vertices.
        for vertex in vertices.iter_mut() {
            if !vertex.boundary {
                // Apply one-ring rule for even vertex.
                if vertex.regular {
                    vertex.child.as_mut().unwrap().position = vertex.weight_one_ring(1.0 / 16.0);
                } else {
                    vertex.child.as_mut().unwrap().position =
                        vertex.weight_one_ring(beta(vertex.valence()));
                }
            } else {
                // Apply boundary rule for even vertex.
                vertex.child.as_mut().unwrap().position = vertex.weight_boundary(1.0 / 8.0);
            }
        }

        // Compute new odd edge vertices.
        let mut edge_vertices: HashMap<SDEdge, SDVertex> = HashMap::new();
        for face in faces.iter() {
            for k in 0..3 {
                // Compute odd vertex on kth edge.
                let edge = SDEdge::new(face.vertices[k].clone(), face.vertices[next(k)].clone());

                let mut initialized = false;
                for (e, v) in &edge_vertices {
                    if e == &edge {
                        initialized = true;
                    }
                }

                if !initialized {
                    // Create and initialize new odd vertex.
                    let mut vertex = SDVertex::new(Point3::default());
                    vertex.regular = true;
                    vertex.boundary = face.faces.get(k).is_none();
                    vertex.start_face = Some(face.children[3].clone());

                    // Apply edge rules to compute new vertex position.
                    if vertex.boundary {
                        vertex.position = 0.5 * edge.vertices[0].position;
                        vertex.position += 0.5 * edge.vertices[1].position;
                    } else {
                        vertex.position = 3.0 / 8.0 * edge.vertices[0].position;
                        vertex.position += 3.0 / 8.0 * edge.vertices[1].position;
                        vertex.position += 1.0 / 8.0
                            * face
                                .other_vertex(&edge.vertices[0], &edge.vertices[1])
                                .position;
                        vertex.position += 1.0 / 8.0
                            * face.faces[k]
                                .other_vertex(&edge.vertices[0], &edge.vertices[1])
                                .position;
                    }

                    new_vertices.push(vertex.clone());
                    edge_vertices.insert(edge, vertex);
                }
            }
        }

        // Update even vertex face.
        for vertex in vertices.iter_mut() {
            let vertex_index = vertex.start_face.as_ref().unwrap().get_vertex_index(vertex);
            vertex.child.as_mut().unwrap().start_face =
                Some(vertex.start_face.as_ref().unwrap().children[vertex_index].clone());
        }

        // Update face neighbors.
        for face in faces.iter_mut() {
            for j in 0..3 {
                // Update children faces for siblings.
                face.children[3].faces[j] = face.children[next(j)].clone();
                face.children[j].faces[next(j)] = face.children[3].clone();

                // Update children faces for neighbor children.
                if let Some(f) = face.faces.get(j) {
                    face.children[j].faces[j] =
                        f.children[f.get_vertex_index(&face.vertices[j])].clone()
                }
                if let Some(f) = face.faces.get(previous(j)) {
                    face.children[j].faces[previous(j)] =
                        f.children[f.get_vertex_index(&face.vertices[j])].clone()
                }
            }
        }

        // Update face vertex.
        for face in faces.iter_mut() {
            for j in 0..3 {
                // Update child vertex to new even vertex.
                face.children[j].vertices[j] = *face.vertices[j].child.as_ref().unwrap().clone();

                // Update child vertex to new odd vertex.
                let mut vertex = None;
                let edge = SDEdge::new(face.vertices[j].clone(), face.vertices[next(j)].clone());
                for (e, v) in &edge_vertices {
                    if e == &edge {
                        vertex = Some(v);
                    }
                }

                let vertex = vertex.unwrap();
                face.children[j].vertices[next(j)] = vertex.clone();
                face.children[next(j)].vertices[j] = vertex.clone();
                face.children[3].vertices[j] = vertex.clone();
            }
        }

        // Prepare for next level of subdivision.
        vertices = new_vertices;
        faces = new_faces;
    }

    // Push vertices to limit surface.
    let mut point_limit: Vec<Point3> = Vec::with_capacity(vertices.len());
    for v in vertices.iter_mut() {
        let point = if v.boundary {
            v.weight_boundary(1.0 / 5.0)
        } else {
            v.weight_one_ring(loop_gamma(v.valence()))
        };
        point_limit.push(point.clone());
        v.position = point;
    }

    // Compute vertex tangents on limit surface.
    let mut normals: Vec<Normal> = Vec::with_capacity(vertices.len());
    let mut point_ring = vec![Point3::default(); 16];
    for vertex in vertices.iter() {
        let mut s = Vec3::default();
        let mut t = Vec3::default();

        let valence = vertex.valence();
        if valence > point_ring.len() as i32 {
            point_ring.resize(valence as usize, Point3::default());
        }

        if !vertex.boundary {
            // Compute tangents of interior face.
            for j in 0..valence {
                s += (2.0 * PI * ((j / valence) as Float)).cos()
                    * Vec3::from(point_ring[j as usize]);
                t += (2.0 * PI * ((j / valence) as Float)).sin()
                    * Vec3::from(point_ring[j as usize]);
            }
        } else {
            // Compute tangents of boundary face.
            s = point_ring[(valence - 1) as usize] - point_ring[0];
            if valence == 2 {
                t = Vec3::from(point_ring[0] + point_ring[1] - 2.0 * vertex.position);
            } else if valence == 3 {
                t = point_ring[1] - vertex.position;
            } else if valence == 4 {
                t = Vec3::from(
                    -1.0 * point_ring[0]
                        + 2.0 * point_ring[1]
                        + 2.0 * point_ring[2]
                        + -1.0 * point_ring[3]
                        + -2.0 * vertex.position,
                );
            } else {
                let theta = PI / ((valence - 1) as Float);
                t = Vec3::from(theta.sin() * (point_ring[0] + point_ring[(valence - 1) as usize]));
                for k in 1..(valence - 1) {
                    let wt = (2.0 * theta.cos() - 2.0) * ((k as Float) * theta).sin();
                    t += Vec3::from(wt * point_ring[k as usize]);
                }
                t = -t;
            }
        }

        normals.push(Normal::from(s.cross(&t)));
    }

    // Create triangle mesh from subdivision mesh.
    todo!();
}
