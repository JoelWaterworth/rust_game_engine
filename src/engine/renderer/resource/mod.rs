use ash::vk;
use ash::version::{DeviceV1_0, V1_0};
use ash::util::*;
use std::sync::Arc;
use std::mem::align_of;
use std::ptr;

use engine::renderer::device::Device;
use engine::renderer::memory::find_memorytype_index;

pub struct Resource {
    device: Arc<Device>,
    pub memory: vk::DeviceMemory,
    pub buffer: vk::Buffer,
    pub descriptor: vk::DescriptorBufferInfo,
    pub size: u64,
    align: Option<u64>
}

impl Resource {
    pub fn create_resource(device: Arc<Device>,
                           usage: vk::BufferUsageFlags,
                           memory_properties: vk::MemoryPropertyFlags,
                           size: usize) -> Self {
        Resource::create_resource_option(device,usage,memory_properties,size,None)
    }

    pub fn create_resource_with_alignment(
        device: Arc<Device>,
        usage: vk::BufferUsageFlags,
        memory_properties: vk::MemoryPropertyFlags,
        size: usize,
        align: usize) -> Self {
        Resource::create_resource_option(device,usage,memory_properties,size,Some(align as u64))
    }

    fn create_resource_option(
                            device: Arc<Device>,
                            usage: vk::BufferUsageFlags,
                            memory_properties: vk::MemoryPropertyFlags,
                            size: usize,
                            align: Option<u64>) -> Self {
        unsafe {
            let buffer_info = vk::BufferCreateInfo {
                s_type: vk::StructureType::BufferCreateInfo,
                p_next: ptr::null(),
                flags: vk::BufferCreateFlags::empty(),
                size: size as u64,
                usage: usage,
                sharing_mode: vk::SharingMode::Exclusive,
                queue_family_index_count: 0,
                p_queue_family_indices: ptr::null(),
            };

            let buffer = device.create_buffer(&buffer_info, None).unwrap();
            let memory_req = device.get_buffer_memory_requirements(buffer);
            let memory_index = find_memorytype_index(&memory_req,
                                                     &device.memory_properties,
                                                     memory_properties)
                .expect("Unable to find suitable memorytype for the index buffer.");

            let allocate_info = vk::MemoryAllocateInfo {
                s_type: vk::StructureType::MemoryAllocateInfo,
                p_next: ptr::null(),
                allocation_size: memory_req.size,
                memory_type_index: memory_index,
            };

            let memory = device.allocate_memory(&allocate_info, None).unwrap();

            device.bind_buffer_memory(buffer, memory, 0).unwrap();

            Resource {
                device: device.clone(),
                memory,
                buffer,
                descriptor: vk::DescriptorBufferInfo {
                    buffer,
                    offset: 0,
                    range: vk::VK_WHOLE_SIZE,
                },
                size: size as u64,
                align
            }
        }
    }

    pub fn map<T>(&self) -> Align<T> {
        unsafe {
            let ptr = self.device
                .map_memory(self.memory,
                                 0,
                                 self.size as u64,
                                 vk::MemoryMapFlags::empty())
                .unwrap();
            match self.align {
                Some(x) => Align::new(ptr, x, self.size as u64),
                None => Align::new(ptr, align_of::<T>() as u64, self.size as u64)
            }
        }
    }

    pub fn unmap(&self) {
        unsafe {
            self.device.unmap_memory(self.memory)
        }
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        unsafe {
            //self.device.unmap_memory(self.memory);
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
}