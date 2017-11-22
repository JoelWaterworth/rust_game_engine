use cgmath::{Matrix4,Vector3,Rad,perspective};
use cgmath::conv::{array4x4};
use cgmath::Point3;

use std::ops::AddAssign;

#[derive(Copy, Clone, Debug)]
pub struct ModelSpace {
    mat: [[f32; 4]; 4]
}

impl ModelSpace {
    pub fn from_location(location: Vector3<f32>) -> ModelSpace {
        ModelSpace::new(array4x4(Matrix4::from_translation(location)))
    }
    pub fn new(mat: [[f32; 4]; 4]) -> ModelSpace {
        ModelSpace{ mat }
    }
}

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
pub struct SMRotation {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl SMRotation {
    pub fn new (x: f32, y: f32, z: f32 ) -> SMRotation {
        SMRotation { x: x, y: y, z: z }
    }
    pub fn default() -> SMRotation {
        SMRotation::new(0.0,0.0,0.0)
    }
}

impl AddAssign for SMRotation {
    fn add_assign(&mut self, _rhs: SMRotation) {
        self.x = self.x + _rhs.x;
        self.y = self.y + _rhs.y;
        self.z = self.z + _rhs.z;
    }
}

#[derive(Clone)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: SMRotation,
    pub scale: Vector3<f32>,
    pub up_vector: Vector3<f32>,
    pub forward_vector: Vector3<f32>,
}

impl Transform {
    pub fn new(position: Vector3<f32>, rotation: SMRotation, scale: Vector3<f32>) -> Transform {
        Transform{position, rotation, scale, up_vector: Vector3::new(0.0, 1.0, 0.0), forward_vector: Vector3::new(0.0, 0.0, -1.0),}
    }
    pub fn to_matrix(&self) ->  Matrix4<f32> {
        (Matrix4::from_angle_y(Rad(self.rotation.y))) * (Matrix4::from_angle_x(Rad(self.rotation.x))) * (Matrix4::from_angle_z(Rad(self.rotation.z))) *
            (Matrix4::from_translation(Vector3::new(self.position.x,self.position.y,self.position.z))) *
            (Matrix4::from_nonuniform_scale(self.scale.x,self.scale.y,self.scale.z))
    }
    pub fn default() -> Transform {
        Transform::new(Vector3::new(0.0, 0.0, 0.0), SMRotation::default(), Vector3::new(1.0, 1.0, 1.0))
    }

    pub fn add_to_rotation(&mut self, rotation: SMRotation) { self.rotation += rotation; }

    pub fn set_location(&mut self, vector: Vector3<f32>) { self.position = vector; }

    pub fn set_rotation(&mut self, rotation: SMRotation) { self.rotation = rotation; }

    pub fn from_position(position: Vector3<f32>) -> Transform {
        Transform::new(position, SMRotation::default(), Vector3::new(1.0, 1.0, 1.0))
    }
}

pub struct Camera {
    pub transform: Transform,
    pub fov: f32,
}

impl Camera {
    pub fn new (transform: Transform, fov: f32) -> Camera {
        Camera { transform: transform.clone(), fov: fov}
    }
    pub fn default() -> Camera {
        Camera::new(Transform::default(), 90.0)
    }
    pub fn mat_transform(&self) -> Matrix4<f32> { self.transform.to_matrix() }

    pub fn mat_perspective(&self, width: u32, height: u32) -> Matrix4<f32> {
        perspective(Rad(self.fov.to_radians()), width as f32 / height as f32, 0.01, 1024.0)
    }

    pub fn look_at(&self) -> Matrix4<f32> {
        let position = Point3::new(self.transform.position.x, self.transform.position.y, self.transform.position.z);

        Matrix4::look_at(position, position + self.transform.forward_vector, self.transform.up_vector)
    }
}