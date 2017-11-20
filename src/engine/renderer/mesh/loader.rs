use tobj;
use std::path::Path;
use std::ffi::OsStr;
use cgmath::{Vector3, Vector2};
use cgmath::InnerSpace;

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tangent: Vector3<f32>,
    pub uv: Vector2<f32>,
}

pub fn load<P: AsRef<OsStr> + ? Sized>(path: &P) -> (Vec<Vertex>, Vec<u32>) {
    let cornell_box = tobj::load_obj(Path::new(path));
    let (models, _) = cornell_box.unwrap();
    let mesh = &models[0].mesh;
    let mut vertices: Vec<Vertex> = Vec::new();
    for x in 0..(mesh.positions.len() / 3) {
        let vertex = Vertex {
            pos: Vector3::from([mesh.positions[x * 3], mesh.positions[x * 3 + 1], mesh.positions[x * 3 + 2]]),
            normal: Vector3::from([mesh.normals[x * 3], mesh.normals[x * 3 + 1], mesh.normals[x * 3 + 2]]),
            tangent: Vector3::from([0.0, 0.0, 0.0]),
            uv: Vector2::from([mesh.texcoords[x * 2], mesh.texcoords[x * 2 + 1]]),
        };
        vertices.push(vertex);
    };
    calculate_tangent(&mut vertices, &mesh.indices);
    (vertices, mesh.indices.clone())
}

//based off http://ogldev.atspace.co.uk/www/tutorial26/tutorial26.html
fn calculate_tangent(vertices: &mut Vec<Vertex>, indices: &Vec<u32>) {
    let mut i = indices.iter();
    loop {
        match i.next() {
            None => break,
            Some(&f) => {
                let s = i.next().unwrap().clone() as usize;
                let t = i.next().unwrap().clone() as usize;
                let v0 = vertices[f.clone() as usize];
                let v1 = vertices[s];
                let v2 = vertices[t];

                let edge1 = v1.pos - v0.pos;
                let edge2 = v2.pos - v0.pos;

                let delta_u1 = v1.uv.x - v0.uv.x;
                let delta_v1 = v1.uv.y - v0.uv.y;
                let delta_u2 = v2.uv.x - v0.uv.x;
                let delta_v2 = v2.uv.y - v0.uv.y;

                let x = 1.0 / (delta_u1 * delta_v2 - delta_u2 * delta_v1);

                let tangent = Vector3::new(
                    x * (delta_v2 * edge1.x - delta_v1 * edge2.x),
                    x * (delta_v2 * edge1.y - delta_v1 * edge2.y),
                    x * (delta_v2 * edge1.z - delta_v1 * edge2.z)
                );

                #[allow(unused_variables)]
                let bitangent = Vector3::new(
                    x * (delta_u2 * edge1.x - delta_u1 * edge2.x),
                    x * (delta_u2 * edge1.y - delta_u1 * edge2.y),
                    x * (delta_u2 * edge1.z - delta_u1 * edge2.z)
                );
                vertices[f as usize].tangent += tangent;
                vertices[s].tangent += tangent;
                vertices[t].tangent += tangent;
            }
        }
    }

    for vertex in vertices.iter_mut() {
        vertex.tangent.normalize();
    }
}