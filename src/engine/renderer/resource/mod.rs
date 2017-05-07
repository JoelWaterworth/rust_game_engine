use ash::vk;
use ash::version::{InstanceV1_0, DeviceV1_0, V1_0};
use std::sync::Arc;
use std::ptr;
use ash::vk::c_void;
use std::mem;

use engine::renderer::device::Device;
use engine::renderer::memory::find_memorytype_index;

pub struct Resource<T> {
    pub buffer: vk::Buffer,
    pub size: usize,
    pub memory: vk::DeviceMemory,
    device: Arc<Device>,
    pub mapped: Box<[T]>,
}

impl<T> Resource<T>
    where T: Clone + Copy + Sized {
    pub fn create_resource(
        device: Arc<Device>,
        usage: vk::BufferUsageFlags,
        memory_properties: vk::MemoryPropertyFlags,
        size: usize,
    ) -> Resource<T> {
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

            let mem_req = device.get_buffer_memory_requirements(buffer);
            let allocate_info = vk::MemoryAllocateInfo {
                s_type: vk::StructureType::MemoryAllocateInfo,
                p_next: ptr::null(),
                allocation_size: mem_req.size,
                memory_type_index: find_memorytype_index(&mem_req,
                                                         &device.memory_properties,
                                                         memory_properties)
                    .expect("Unable to find suitable memorytype for the vertex buffer.")
            };

            let memory = device.allocate_memory(&allocate_info, None)
                .unwrap();
            device.bind_buffer_memory(buffer, memory, 0).unwrap();

            let mapped = Box::from_raw(device
                .map_memory::<T>(memory,
                                 0,
                                 size as u64,
                                 vk::MemoryMapFlags::empty()).unwrap());

            Resource { buffer: buffer, memory: memory, device: device, size: size, mapped: mapped }
        }
    }

    pub fn copy_data(&mut self, data: &Vec<T>) {
        self.mapped.copy_from_slice(data);
    }

    pub fn get_descriptor_buffer_info(&self) -> vk::DescriptorBufferInfo {
        vk::DescriptorBufferInfo {
            buffer: self.buffer,
            offset: 0,
            range: self.size as u64,
        }
    }

    pub fn flush(&self) {
        unsafe {
            let memory_range = vk::MappedMemoryRange {
                s_type: vk::StructureType::MappedMemoryRange,
                p_next: ptr::null(),
                memory: self.memory,
                offset: 0,
                size: self.size as u64,
            };
            self.device.fp_v1_0().flush_mapped_memory_ranges(
                self.device.handle(),
                1,
                &memory_range);
        }
    }
}

impl<T> Drop for Resource<T> {
    fn drop(&mut self) {
        unsafe {
            self.device.unmap_memory(self.memory);
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
}
