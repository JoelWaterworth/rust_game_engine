use camera::{Transform, Camera};
use std::cell::RefCell;
use std::rc::Rc;

use scene_object::Actor;
use renderer::resource::ResourceManager;

struct World {
    static_actors: Vec<Rc<RefCell<Actor>>>,
    active_camera: Camera,
    resource_manager: ResourceManager
}

impl World {
    pub fn new(camera: Camera) -> Self {
        Self {static_actors: Vec::new(), active_camera: camera, resource_manager: ResourceManager::new()}
    }
    pub fn add<A: Sized + Actor + 'static>(&mut self, actor_fn: fn() -> A, transform: Transform) {
        self.static_actors.push(Rc::new(RefCell::new(actor_fn())));
    }
    //pub fn get_static_actors_mat4(&self) ->
}