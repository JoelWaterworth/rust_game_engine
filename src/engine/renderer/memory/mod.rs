use ash::vk;
use ash::version::{InstanceV1_0, DeviceV1_0, V1_0};
use std::ptr;
use engine::renderer::device::Device;
use std::sync::Arc;

pub fn find_memorytype_index(memory_req: &vk::MemoryRequirements,
                             memory_prop: &vk::PhysicalDeviceMemoryProperties,
                             flags: vk::MemoryPropertyFlags)
                             -> Option<u32> {
    // Try to find an exactly matching memory flag
    let best_suitable_index =
        find_memorytype_index_f(memory_req,
                                memory_prop,
                                flags,
                                |property_flags, flags| property_flags == flags);
    if best_suitable_index.is_some() {
        return best_suitable_index;
    }
    // Otherwise find a memory flag that works
    find_memorytype_index_f(memory_req,
                            memory_prop,
                            flags,
                            |property_flags, flags| property_flags & flags == flags)
}

pub fn find_memorytype_index_f<F: Fn(vk::MemoryPropertyFlags, vk::MemoryPropertyFlags) -> bool>
(memory_req: &vk::MemoryRequirements,
 memory_prop: &vk::PhysicalDeviceMemoryProperties,
 flags: vk::MemoryPropertyFlags,
 f: F)
 -> Option<u32> {
    let mut memory_type_bits = memory_req.memory_type_bits;
    for (index, ref memory_type) in memory_prop.memory_types.iter().enumerate() {
        if memory_type_bits & 1 == 1 {
            if f(memory_type.property_flags, flags) {
                return Some(index as u32);
            }
        }
        memory_type_bits = memory_type_bits >> 1;
    }
    None
}

pub unsafe fn create_allocated_buffer(device: &Arc<Device>,
                           size: vk::DeviceSize,
                           usage: vk::BufferUsageFlags,
                           properties: vk::MemoryPropertyFlags) -> (vk::Buffer, vk::DeviceMemory){
    let buffer_info = vk::BufferCreateInfo {
        s_type: vk::StructureType::BufferCreateInfo,
        p_next: ptr::null(),
        flags: vk::BufferCreateFlags::empty(),
        size: size,
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
                                                 properties)
            .expect("Unable to find suitable memorytype for the vertex buffer.")
    };

    let memory = device
        .allocate_memory(&allocate_info, None)
        .unwrap();
    device.bind_buffer_memory(buffer, memory, 0).unwrap();
    (buffer, memory)
}
