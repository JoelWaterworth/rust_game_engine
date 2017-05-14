use ash::vk;
pub use ash::version::{V1_0, InstanceV1_0, DeviceV1_0, EntryV1_0};

use std::mem;
use std::ptr;
use std::sync::Arc;

use camera::MVP;
use engine::renderer::resource::Resource;
use engine::renderer::device::Device;

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
    dynamic: Resource,

}

impl UniformBuffer {
    pub fn init<T: Clone + Copy + Sized>(device: Arc<Device>, data: T) -> UniformBuffer {
        unsafe {
            let dynamic = Resource::create_resource(
                device,
                vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT,
                vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT,
                mem::size_of_val(&data));

            {
                let mut map = dynamic.map::<T>();

                map.copy_from_slice(&[data]);

                dynamic.unmap();
            }

            UniformBuffer { dynamic }
        }
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
    dynamic: Resource,
    device: Arc<Device>,
    size: usize,
    align: usize,
}

impl DynamicUniformBuffer {
    pub fn init<T: Clone + Copy + Sized>(device: Arc<Device>, data: Vec<T>) -> DynamicUniformBuffer {
        unsafe {
            let ubo_alignment = device.device_properties.limits.min_uniform_buffer_offset_alignment;
            println!("ubo_alignment {}", ubo_alignment);
            let type_size = mem::size_of::<MVP>() as u64;
            println!("type_size {}", type_size);
            let alignment = if (type_size % ubo_alignment) > 0 { ubo_alignment } else { 0 };
            let dynamic_aligment = ((type_size / ubo_alignment) * ubo_alignment + alignment) as usize;
            println!("dynamic_aligment {}", dynamic_aligment);
            let buffer_size = data.len() * dynamic_aligment;

            let dynamic = Resource::create_resource(
                device.clone(),
                vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT,
                vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT,
                buffer_size);

            {
                let mut map = dynamic.map::<T>();

                println!("{}", map.len());

                map.copy_from_slice(&data);

                dynamic.unmap();
            }

            DynamicUniformBuffer {
                dynamic: dynamic,
                device: device,
                size: buffer_size,
                align: ubo_alignment as usize
            }
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
