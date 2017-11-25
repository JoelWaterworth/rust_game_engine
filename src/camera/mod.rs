use cgmath::{Matrix4,Vector3,perspective, Euler, Deg};
use cgmath::conv::{array4x4};
use cgmath::Point3;

pub type Mat4 = [[f32; 4]; 4];

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VP {
    projection: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
}

impl VP {
    pub fn from_camera(camera: &Camera, width: u32, height: u32) -> Self {
        let world_matrix  = camera.look_at();
        let projection                 = camera.mat_perspective(width, height) * world_matrix;

        Self::new(array4x4(projection), array4x4(world_matrix))
    }

    pub fn new(projection: [[f32; 4]; 4], view: [[f32; 4]; 4]) -> Self {
        Self{ projection, view}
    }
}

#[derive(Clone)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Euler<Deg<f32>>,
    pub scale: Vector3<f32>,
    pub up_vector: Vector3<f32>,
    pub forward_vector: Vector3<f32>,
}

impl Transform {
    pub fn new(position: Vector3<f32>, rotation: Euler<Deg<f32>>, scale: Vector3<f32>) -> Transform {
        Transform{position, rotation, scale, up_vector: Vector3::new(0.0, 1.0, 0.0), forward_vector: Vector3::new(0.0, 0.0, -1.0),}
    }
    pub fn to_matrix(&self) ->  Matrix4<f32> {
        Matrix4::from_translation(Vector3::new(self.position.x,self.position.y,self.position.z)) *
            Matrix4::from(self.rotation) *
                Matrix4::from_nonuniform_scale(self.scale.x,self.scale.y,self.scale.z)
    }

    pub fn to_mat4(&self) -> Mat4 {
        array4x4(self.to_matrix())
    }

    pub fn default() -> Transform {
        Transform::new(Vector3::new(0.0, 0.0, 0.0), Euler::new(Deg(0.0), Deg(0.0), Deg(0.0)), Vector3::new(1.0, 1.0, 1.0))
    }

    pub fn set_location(&mut self, vector: Vector3<f32>) { self.position = vector; }

    pub fn from_position(position: Vector3<f32>) -> Transform {
        Transform::new(position, Euler::new(Deg(0.0), Deg(0.0), Deg(0.0)), Vector3::new(1.0, 1.0, 1.0))
    }
}

pub struct Camera {
    pub transform: Transform,
    pub fov: f32,
}

impl Camera {
    pub fn new (transform: Transform, fov: f32) -> Camera {
        Camera { transform: transform.clone(), fov}
    }
    pub fn default() -> Camera {
        Camera::new(Transform::default(), 90.0)
    }
    pub fn mat_transform(&self) -> Matrix4<f32> { self.transform.to_matrix() }

    pub fn mat_perspective(&self, width: u32, height: u32) -> Matrix4<f32> {
        perspective(Deg(self.fov), width as f32 / height as f32, 0.01, 1024.0)
    }

    pub fn look_at(&self) -> Matrix4<f32> {
        let position = Point3::new(self.transform.position.x, self.transform.position.y, self.transform.position.z);

        Matrix4::look_at(position, position + self.transform.forward_vector, self.transform.up_vector)
    }
}