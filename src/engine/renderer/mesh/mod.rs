use ash::vk;
pub use ash::version::{V1_0, InstanceV1_0, DeviceV1_0, EntryV1_0};
use ash::util::*;

use std::ops::Drop;
use std::sync::Arc;
use std::u32;
use std::u64;
use std::ptr;
use std::mem;
use std::mem::align_of;
use std::ffi::OsStr;

use engine::renderer::device::Device;
use engine::renderer::{find_memorytype_index};
use engine::renderer::vk_commands::record_submit_commandbuffer;
use engine::renderer::memory::create_allocated_buffer;

mod loader;
use self::loader::load;
pub use self::loader::Vertex;

pub struct Mesh {
    pub device: Arc<Device>,
    pub memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub vertex_buffer: vk::Buffer,
    pub index_buffer_len: u32,
    pub index_offset: u64,
    pub vertex_offset: u64
}

//TODO: have the index and vertex data be inside the same buffer
impl Mesh {
    pub fn new<P: AsRef<OsStr> + ?Sized>(device: Arc<Device>, path: &P, command_buffer: vk::CommandBuffer)-> Mesh { unsafe {
        let (vertices, index_data) = load(path);
        let index_data_size = (mem::size_of::<u32>() * index_data.len()) as u64;
        //let index_offset = 0;
        let vertex_data_size = (mem::size_of::<Vertex>() * vertices.len()) as u64;
        //let vertex_offset = index_data_size;

        let (staging_index_buffer, staging_index_memory) =
            create_allocated_buffer(&device,
                                    index_data_size,
                                    vk::BUFFER_USAGE_TRANSFER_SRC_BIT,
                                    vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT);

        let index_ptr = device
            .map_memory(staging_index_memory,
                               0,
                               index_data_size,
                               vk::MemoryMapFlags::empty())
            .unwrap();
        let mut index_slice = Align::new(index_ptr, align_of::<u32>() as u64, index_data_size);
        index_slice.copy_from_slice(&index_data);
        device.unmap_memory(staging_index_memory);

        let (staging_vertex_buffer, staging_vertex_memory) =
            create_allocated_buffer(&device,
                                    vertex_data_size,
                                    vk::BUFFER_USAGE_TRANSFER_SRC_BIT,
                                    vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT);

        let vertex_ptr = device
            .map_memory(staging_vertex_memory,
                                  0,
                                  vertex_data_size,
                                  vk::MemoryMapFlags::empty())
            .unwrap();
        let mut vertex_slice = Align::new(vertex_ptr, align_of::<f32>() as u64, vertex_data_size);
        vertex_slice.copy_from_slice(&vertices);
        device.unmap_memory(staging_vertex_memory);

        let (index_buffer, vertex_buffer, memory) =
            multi_buffer_allocation(&device,
                                    index_data_size, vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_INDEX_BUFFER_BIT,
                                    vertex_data_size, vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_VERTEX_BUFFER_BIT,
                                    vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT);

        record_submit_commandbuffer(&device,
                                    command_buffer,
                                    &[vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT],
                                    &[],
                                    &[],
                                    |cmd| {
                                        device.cmd_copy_buffer(cmd, staging_index_buffer, index_buffer,
                                                               &[vk::BufferCopy {
                                                                   src_offset: 0,
                                                                   dst_offset: 0,
                                                                   size: index_data_size
                                                               }]);
                                        device.cmd_copy_buffer(cmd, staging_vertex_buffer, vertex_buffer,
                                                               &[vk::BufferCopy {
                                                                   src_offset: 0,
                                                                   dst_offset: 0,
                                                                   size: vertex_data_size
                                                               }]);
                                    });

        device.free_memory(staging_index_memory, None);
        device.destroy_buffer(staging_index_buffer, None);

        device.free_memory(staging_vertex_memory, None);
        device.destroy_buffer(staging_vertex_buffer, None);

        Mesh {
            device: device.clone(),
            memory,
            index_buffer,
            vertex_buffer,

            index_buffer_len: index_data.len() as u32,
            index_offset: 0,
            vertex_offset: 0
        }
    }}

    pub unsafe fn draw(&self, command_buffer: vk::CommandBuffer) {
        self.device.cmd_bind_vertex_buffers(
            command_buffer, 0, &[self.vertex_buffer], &[self.vertex_offset]);

        self.device.cmd_bind_index_buffer(
            command_buffer,
            self.index_buffer,
            self.index_offset,
            vk::IndexType::Uint32);

        self.device.cmd_draw_indexed(command_buffer,
                                     self.index_buffer_len,
                                     1,
                                     0,
                                     self.vertex_offset as i32,
                                     1);
    }
}

unsafe fn copy_buffer(device: &Arc<Device>, command_buffer: vk::CommandBuffer, source: vk::Buffer, destination: vk::Buffer, region: vk::BufferCopy) {
    record_submit_commandbuffer(&device,
                                command_buffer,
                                &[vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT],
                                &[],
                                &[],
                                |command_buffer| {
                                    device.cmd_copy_buffer(command_buffer, source, destination, &[region]);
                                });
}

unsafe fn multi_buffer_allocation(device: &Arc<Device>,
                                  index_size: vk::DeviceSize, index_usage: vk::BufferUsageFlags,
                                  vertex_size: vk::DeviceSize, vertex_usage: vk::BufferUsageFlags,
                                  properties: vk::MemoryPropertyFlags) -> (vk::Buffer, vk::Buffer, vk::DeviceMemory) {
    let index_buffer_info = vk::BufferCreateInfo {
        s_type: vk::StructureType::BufferCreateInfo,
        p_next: ptr::null(),
        flags: vk::BufferCreateFlags::empty(),
        size: index_size,
        usage: index_usage,
        sharing_mode: vk::SharingMode::Exclusive,
        queue_family_index_count: 0,
        p_queue_family_indices: ptr::null(),
    };
    let index_buffer = device.create_buffer(&index_buffer_info, None).unwrap();

    let vertex_buffer_info = vk::BufferCreateInfo {
        s_type: vk::StructureType::BufferCreateInfo,
        p_next: ptr::null(),
        flags: vk::BufferCreateFlags::empty(),
        size: vertex_size,
        usage: vertex_usage,
        sharing_mode: vk::SharingMode::Exclusive,
        queue_family_index_count: 0,
        p_queue_family_indices: ptr::null(),
    };
    let vertex_buffer = device.create_buffer(&vertex_buffer_info, None).unwrap();

    let index_mem_req = device.get_buffer_memory_requirements(index_buffer);
    let vertex_mem_req = device.get_buffer_memory_requirements(vertex_buffer);
    let allocate_info = vk::MemoryAllocateInfo {
        s_type: vk::StructureType::MemoryAllocateInfo,
        p_next: ptr::null(),
        allocation_size: (index_mem_req.size + vertex_mem_req.size),
        memory_type_index: find_memorytype_index(&index_mem_req,
                                                 &device.memory_properties,
                                                 properties)
            .expect("Unable to find suitable memorytype for the vertex buffer.")
    };
    let memory = device
        .allocate_memory(&allocate_info, None)
        .unwrap();
    device.bind_buffer_memory(index_buffer, memory, 0).unwrap();
    device.bind_buffer_memory(vertex_buffer, memory, index_mem_req.size).unwrap();
    (index_buffer, vertex_buffer, memory)
}

impl Drop for Mesh {
    fn drop(&mut self) { unsafe {
        //self.device.free_memory(self.vertex_memory, None);
        self.device.free_memory(self.memory, None);
        self.device.destroy_buffer(self.vertex_buffer, None);
        self.device.destroy_buffer(self.index_buffer, None);
    }}
}
