use ash::vk;
pub use ash::version::{V1_0, InstanceV1_0, DeviceV1_0, EntryV1_0};

use std::ptr;
use std::sync::Arc;
use std::u64;

use engine::renderer::device::Device;

pub struct Pool {
    device: Arc<Device>,
    pub pool: vk::CommandPool,
    pub draw_command_buffer: Vec<vk::CommandBuffer>,
    pub setup_command_buffer: vk::CommandBuffer,
    pub g_buffer_setup: vk::CommandBuffer,
    pub off_screen_command_buffer: vk::CommandBuffer,
}

impl Pool {
    pub fn init(device: Arc<Device>, draw_buffer_num: u32) -> Pool { unsafe {
        let pool_create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::CommandPoolCreateInfo,
            p_next: ptr::null(),
            flags: vk::COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,
            queue_family_index: device.queue_family_index,
        };
        let pool = device.create_command_pool(&pool_create_info, None).unwrap();
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::CommandBufferAllocateInfo,
            p_next: ptr::null(),
            command_buffer_count: 3,
            command_pool: pool,
            level: vk::CommandBufferLevel::Primary,
        };
        let command_buffers = device.allocate_command_buffers(&command_buffer_allocate_info)
            .unwrap();
        let setup_command_buffer = command_buffers[0];
        let off_screen_command_buffer = command_buffers[1];
        let g_buffer_setup = command_buffers[2];

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::CommandBufferAllocateInfo,
            p_next: ptr::null(),
            command_buffer_count: draw_buffer_num,
            command_pool: pool,
            level: vk::CommandBufferLevel::Primary,
        };
        let draw_command_buffers = device.allocate_command_buffers(&command_buffer_allocate_info)
            .unwrap();

        Pool {device,
            pool,
            draw_command_buffer: draw_command_buffers,
            setup_command_buffer,
            g_buffer_setup,
            off_screen_command_buffer}
    } }
}

impl Drop for Pool {
    fn drop(&mut self) { unsafe {
        self.device.destroy_command_pool(self.pool, None);
    } }
}

pub fn record_off_screen<F: FnOnce(vk::CommandBuffer)>(device: &Arc<Device>,
                                                       command_buffer: vk::CommandBuffer,
                                                       f: F) {unsafe {
    let command_buffer_begin_info = vk::CommandBufferBeginInfo {
        s_type: vk::StructureType::CommandBufferBeginInfo,
        p_next: ptr::null(),
        p_inheritance_info: ptr::null(),
        flags: vk::COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT
    };
    device.begin_command_buffer(command_buffer, &command_buffer_begin_info).expect("Begin commandbuffer");
    f(command_buffer);
    device.end_command_buffer(command_buffer).expect("End commandbuffer");
}}

pub fn record_submit_commandbuffer<T, F: FnOnce(vk::CommandBuffer) -> T>(device: &Arc<Device>,
                                                                 command_buffer: vk::CommandBuffer,
                                                                 wait_mask: &[vk::PipelineStageFlags],
                                                                 wait_semaphores: &[vk::Semaphore],
                                                                 signal_semaphores: &[vk::Semaphore],
                                                                 f: F) -> T{
    unsafe {
        device.reset_command_buffer(command_buffer,
                                    vk::COMMAND_BUFFER_RESET_RELEASE_RESOURCES_BIT)
            .expect("Reset command buffer failed.");
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::CommandBufferBeginInfo,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
        };
        device.begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .expect("Begin commandbuffer");
        let val = f(command_buffer);
        device.end_command_buffer(command_buffer).expect("End commandbuffer");
        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FenceCreateInfo,
            p_next: ptr::null(),
            flags: vk::FenceCreateFlags::empty(),
        };
        let submit_fence = device.create_fence(&fence_create_info, None)
            .expect("Create fence failed.");
        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SubmitInfo,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_mask.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &command_buffer,
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        };
        device.queue_submit(device.queue, &[submit_info], submit_fence)
            .expect("queue submit failed.");
        device.wait_for_fences(&[submit_fence], true, u64::MAX)
            .expect("Wait for fence failed.");
        device.destroy_fence(submit_fence, None);
        val
    }
}