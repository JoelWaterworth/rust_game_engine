use ash::vk;
pub use ash::version::{V1_0, InstanceV1_0, DeviceV1_0, EntryV1_0};

use std::mem;
use alloc::heap::{allocate, deallocate};
use std::ptr;
use std::sync::Arc;
use std::rc::Rc;

use camera::MVP;
use engine::renderer::resource::Resource;
use engine::renderer::memory::*;
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

pub struct UniformBuffer<T> {
    device: Arc<Device>,
    pub memory: vk::DeviceMemory,
    pub buffer: vk::Buffer,
    pub descriptor: vk::DescriptorBufferInfo,
    data: T,
}

impl<T> UniformBuffer<T>
    where T: Clone + Copy + Sized {
    pub fn init(device: Arc<Device>, data: T) -> UniformBuffer<T> { unsafe {

        let buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BufferCreateInfo,
            p_next: ptr::null(),
            flags: vk::BufferCreateFlags::empty(),
            size: mem::size_of_val(&data) as u64,
            usage: vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT,
            sharing_mode: vk::SharingMode::Exclusive,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
        };

        let buffer = device.create_buffer(&buffer_info, None).unwrap();
        let memory_req = device.get_buffer_memory_requirements(buffer);
        let memory_index = find_memorytype_index(&memory_req,
                                                 &device.memory_properties,
                                                 vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT)
            .expect("Unable to find suitable memorytype for the index buffer.");

        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MemoryAllocateInfo,
            p_next: ptr::null(),
            allocation_size: memory_req.size,
            memory_type_index: memory_index,
        };

        let memory = device.allocate_memory(&allocate_info, None).unwrap();

        device.bind_buffer_memory(buffer, memory, 0).unwrap();

        let mut uniform_buffer = UniformBuffer {
            device: device.clone(),
            memory: memory,
            buffer: buffer,
            descriptor: vk::DescriptorBufferInfo {
                buffer: buffer,
                offset: 0,
                range: buffer_info.size,
            },
            data: data,
        };
        uniform_buffer.update(data);
        uniform_buffer
    }}

    pub fn update(&mut self, data: T) { unsafe {
        let slice = self.device
            .map_memory::<T>(self.memory,
                             0,
                             mem::size_of_val(&data) as u64,
                             vk::MemoryMapFlags::empty())
            .unwrap();
        slice.copy_from_slice(&[data]);
        self.device.unmap_memory(self.memory);
    }}
}

impl<T> Drop for UniformBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            self.device.free_memory(self.memory, None);
            self.device.destroy_buffer(self.buffer, None);
        }
    }
}

impl<T> Uniform for UniformBuffer<T> {
    fn get_descriptor_type(&self) -> vk::DescriptorType {
        vk::DescriptorType::UniformBuffer
    }
    fn buffer_info(&self) -> *const vk::DescriptorBufferInfo {
        &self.descriptor
    }
}

pub struct DynamicUniformBuffer<T> {
    dynamic: Resource<T>,
    //pub descriptor: vk::DescriptorBufferInfo,
    device: Arc<Device>,
    alloc: *mut u8,
    size: usize,
    align: usize,
}

impl<T> DynamicUniformBuffer<T>
where T: Clone + Copy + Sized {
    pub fn init(device: Arc<Device>, data: &Vec<T>) -> DynamicUniformBuffer<T> { unsafe {
        let ubo_alignment = device.device_properties.limits.min_uniform_buffer_offset_alignment;
        println!("ubo_alignment {}", ubo_alignment);
        let type_size = mem::size_of::<MVP>() as u64;
        println!("type_size {}", type_size);
        let alignment = if ( type_size % ubo_alignment) > 0 {ubo_alignment} else {0};
        let dynamic_aligment = ((type_size / ubo_alignment) * ubo_alignment + alignment) as usize;
        let buffer_size = data.len() * dynamic_aligment;
        let alloc = allocate(buffer_size as usize, alignment as usize);
	println!("DynamicUniformBuffer Resource::create_resource begin");

        let dynamic = Resource::create_resource(
            device.clone(),
            vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT,
            vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT,
            buffer_size);

        let mut x =
            DynamicUniformBuffer {
                dynamic: dynamic,
                device: device,

                alloc: alloc,
                size: buffer_size,
                align: ubo_alignment as usize};
        x.update(data);
        x
    }}

    pub fn update(&mut self, data: &Vec<T>) { unsafe {
    	self.dynamic.copy_data(data);
    	self.dynamic.flush();
    }}
}

impl<T> Drop for DynamicUniformBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            deallocate(self.alloc, self.size, self.align);
            }}
}

impl<T> Uniform for DynamicUniformBuffer<T>
    where T: Clone + Copy + Sized {
    fn get_descriptor_type(&self) -> vk::DescriptorType {
        vk::DescriptorType::UniformBuffer
    }
    fn buffer_info(&self) -> *const vk::DescriptorBufferInfo {
        &self.dynamic.get_descriptor_buffer_info()
    }
}
