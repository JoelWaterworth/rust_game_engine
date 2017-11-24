use ash::vk;
pub use ash::version::{V1_0, InstanceV1_0, DeviceV1_0, EntryV1_0};

use std::mem;
use std::ptr;
use std::sync::Arc;
use std::fmt::Debug;

use renderer::resource::DyanimicResource;
use renderer::device::Device;

pub trait Uniform {
    fn get_descriptor_type(&self) -> vk::DescriptorType;
    fn image_info(&self) -> *const vk::DescriptorImageInfo {
        ptr::null()
    }
    fn buffer_info(&self) -> *const vk::DescriptorBufferInfo {
        ptr::null()
    }
    fn texel_buffer_view(&self) -> *const vk::BufferView {
        ptr::null()
    }
}

pub struct UniformBuffer {
    dynamic: DyanimicResource,
    alignment: u64,
}

impl UniformBuffer {
    pub fn init_with_align<T: Clone + Copy + Sized>(device: Arc<Device>, data: T, alignment: u64) -> UniformBuffer {
        let dynamic = DyanimicResource::create_resource(
                                                device,
                                                vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT,
                                                vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT,
                                                mem::size_of_val(&data));

        let mut map = dynamic.map::<T>();
        map.copy_from_slice(&[data]);
        dynamic.unmap();

        UniformBuffer { dynamic , alignment}
    }
    pub fn init<T: Clone + Copy + Sized>(device: Arc<Device>, data: T) -> UniformBuffer {
        UniformBuffer::init_with_align(device, data, mem::size_of_val(&data) as u64)
    }
}

impl Uniform for UniformBuffer {
    fn get_descriptor_type(&self) -> vk::DescriptorType {
        vk::DescriptorType::UniformBuffer
    }
    fn buffer_info(&self) -> *const vk::DescriptorBufferInfo {
        &self.dynamic.descriptor
    }
}


pub struct DynamicUniformBuffer {
    dynamic: DyanimicResource,
    device: Arc<Device>,
    size: usize,
    pub align: u32,
}

impl DynamicUniformBuffer {
    pub fn init<T: Clone + Copy + Sized + Debug>(device: Arc<Device>, data: Vec<T>) -> DynamicUniformBuffer {

        let ubo_alignment = device.device_properties.limits.min_uniform_buffer_offset_alignment;
        let type_size = mem::size_of::<T>() as u64;
        let alignment = if (type_size % ubo_alignment) > 0 { ubo_alignment } else { 0 };
        let dynamic_aligment = ((type_size / ubo_alignment) * ubo_alignment + alignment) as usize;
        let buffer_size = data.len() * dynamic_aligment;

        let dynamic = DyanimicResource::create_resource_with_alignment(
            device.clone(),
            vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT,
            vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT,
            buffer_size,
            dynamic_aligment);

        let mut map = dynamic.map::<T>();
        map.copy_from_slice(&data);
        dynamic.unmap();

        DynamicUniformBuffer {
            dynamic,
            device,
            size: buffer_size,
            align: ubo_alignment as u32
        }
    }
}

impl Uniform for DynamicUniformBuffer {
    fn get_descriptor_type(&self) -> vk::DescriptorType {
        vk::DescriptorType::UniformBufferDynamic
    }
    fn buffer_info(&self) -> *const vk::DescriptorBufferInfo {
        &self.dynamic.descriptor
    }
}
