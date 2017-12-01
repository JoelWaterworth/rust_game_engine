use camera::Transform;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::path::Path;
use std::sync::Arc;
use renderer::mesh::Mesh;
use renderer::shader::Material;

pub trait Actor {
    fn can_update(&self) -> bool {false}
    fn can_receive_inputs(&self) -> bool {false}
    fn is_static(&self) -> bool {false}
    fn get_actor_data(&self) -> &ActorData;
    fn get_mut_actor_data(&mut self) -> &mut ActorData;
    fn get_transform(&self) -> &Transform {
        &self.get_actor_data().transform
    }

    fn set_transform(&mut self, transform: Transform) {
        self.get_mut_actor_data().transform = transform;
    }
}

pub struct ActorData {
    components: HashMap<String, Rc<RefCell<Component>>>,
    transform: Transform,
}

impl ActorData {
    pub fn new() -> Self {
        ActorData {components: HashMap::new(), transform: Transform::default()}
    }

    pub fn add_component<C: Component + Sized + 'static>(&mut self, name: String, component: C) {
        self.components.insert(name, Rc::new(RefCell::new(component)));
    }
}

pub trait Component {
    fn can_update(&self) -> bool {false}
    fn get_component_data(&self) -> &ComponentData;
    fn get_mut_component_data(&mut self) -> &mut ComponentData;
    fn get_transform(&self) -> &Transform {
        &self.get_component_data().transform
    }

    fn set_transform(&mut self, transform: Transform) {
        self.get_mut_component_data().transform = transform;
    }
}

pub struct ComponentData {
    transform: Transform,
}

pub struct StaticMesh {
    component_data: ComponentData,
    mesh: Arc<Mesh>,
    shader: Arc<Material>,

}

struct EmptyActor {
    actor_data: ActorData
}

impl EmptyActor {
    pub fn new() -> Self {
        Self {actor_data: ActorData::new()}
    }
}

impl Actor for EmptyActor {
    fn get_actor_data(&self) -> &ActorData {
        &self.actor_data
    }
    fn get_mut_actor_data(&mut self) -> &mut ActorData {
        &mut self.actor_data
    }
}

impl Component for StaticMesh {
    fn get_component_data(&self) -> &ComponentData {
        &self.component_data
    }
    fn get_mut_component_data(&mut self) -> &mut ComponentData {
        &mut self.component_data
    }
}