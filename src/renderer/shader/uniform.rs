use ash::vk;
pub use ash::version::{V1_0, InstanceV1_0, DeviceV1_0, EntryV1_0};
use ash::util::*;

use std::mem;
use std::mem::align_of;
use std::ptr;
use std::sync::Arc;
use std::fmt::Debug;

use renderer::resource::DyanimicResource;
use renderer::memory::create_allocated_buffer;
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

pub struct NewUniformBuffer {
    buffer: vk::Buffer,
    descriptor: vk::DescriptorBufferInfo,
    memory: vk::DeviceMemory,
    device: Arc<Device>,
}

impl NewUniformBuffer {
    pub fn init<T: Clone + Copy + Sized + Debug>(device: Arc<Device>, data: T) -> Self { unsafe {
        let size = mem::size_of_val(&data) as u64;
        let (buffer, memory) =
            create_allocated_buffer(&device,
                                    size,
                                    vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT,
                                    vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT);
        let memory_ptr = device
            .map_memory(memory,
                        0,
                        size,
                        vk::MemoryMapFlags::empty())
            .unwrap();
        let mut uniform_slice = Align::new(memory_ptr, align_of::<T>() as u64, size);
        uniform_slice.copy_from_slice(&[data]);
        device.unmap_memory(memory);

        Self {
            device: device.clone(),
            buffer,
            memory,
            descriptor: vk::DescriptorBufferInfo {
                buffer,
                offset: 0,
                range: vk::VK_WHOLE_SIZE,
            },
        }
    }}
}

impl Uniform for NewUniformBuffer {
    fn get_descriptor_type(&self) -> vk::DescriptorType {
        vk::DescriptorType::UniformBuffer
    }
    fn buffer_info(&self) -> *const vk::DescriptorBufferInfo {
        &self.descriptor
    }
}

impl Drop for NewUniformBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
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
