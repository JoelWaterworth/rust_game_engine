use ash;
use ash::vk;
use ash::vk::types;
use ash::vk::cmds::InstanceFnV1_0;
use std::default::Default;
use std::ptr;
use std::ffi::{CString, CStr};

use ash::Entry;
pub use ash::version::{V1_0, InstanceV1_0, DeviceV1_0, EntryV1_0};
#[allow(unused_imports)]
use ash::extensions::{Surface, DebugReport, Win32Surface, XlibSurface};
use std::ops::Drop;

use std::sync::Arc;

use winit;
use std::u32;
use std::u64;
use libc;
use camera::*;

use cgmath::{Vector3};

mod surface;
mod shader;
mod mesh;
mod device;
mod memory;
mod texture;
mod vk_commands;
mod g_buffer;
mod resource;

use engine::renderer::memory::*;
use engine::renderer::vk_commands::{Pool, record_submit_commandbuffer};
use engine::renderer::mesh::Mesh;
use engine::renderer::device::{Device};
use engine::renderer::shader::{Shader, UniformDescriptor};
use engine::renderer::shader::uniform::{DynamicUniformBuffer};
use engine::renderer::surface::*;
use engine::renderer::texture::*;
use engine::renderer::g_buffer::GBuffer;

pub struct Instance {
    pub entry: Entry<V1_0>,
    pub handle: ash::Instance<V1_0>
}

impl Instance {
    fn init(engine_name: &str, app_name: &str)-> Instance {
        let entry = Entry::new().unwrap();

        let app_name = CString::new(app_name).unwrap();
        let raw_app_name = app_name.as_ptr();

        let engine_name = CString::new(engine_name).unwrap();
        let raw_engine_name = engine_name.as_ptr();

        let app_info = vk::ApplicationInfo {
            p_application_name: raw_app_name,
            s_type: vk::StructureType::ApplicationInfo,
            p_next: ptr::null(),
            application_version: 0,
            p_engine_name: raw_engine_name,
            engine_version: 0,
            api_version: vk_make_version!(1, 0, 65),
        };

        let layer_names = [CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];
        let layers_names_raw: Vec<*const i8> = layer_names.iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();
        let extension = get_instance_extensions();

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::InstanceCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            p_application_info: &app_info,
            pp_enabled_layer_names: layers_names_raw.as_ptr(),
            enabled_layer_count: layers_names_raw.len() as u32,
            pp_enabled_extension_names: extension.as_ptr(),
            enabled_extension_count: extension.len() as u32,
        };

        let instance: ash::Instance<V1_0> = unsafe {
            entry.create_instance(&create_info, None)
                .expect("Instance creation error")
        };

        Instance{entry,
            handle: instance}
    }
}

impl Drop for Instance {
    fn drop(&mut self) { unsafe {
        self.handle.destroy_instance(None);
    }}
}

impl InstanceV1_0 for Instance {
    type Fp = V1_0;
    fn handle(&self) -> types::Instance{
        self.handle.handle()
    }
    fn fp_v1_0(&self) -> &InstanceFnV1_0{
        self.handle.fp_v1_0()
    }
}

fn get_instance_layers() -> Vec<*const i8> {
    let layer_names = [CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];
    let layers_names_raw: Vec<*const i8> = layer_names.iter()
        .map(|raw_name| raw_name.as_ptr())
        .collect();
    layers_names_raw
}

#[cfg(all(windows))]
fn get_instance_extensions() -> Vec<*const i8> {
    vec![Surface::name().as_ptr(),
         Win32Surface::name().as_ptr(),
         DebugReport::name().as_ptr()]
}

#[cfg(all(unix, not(target_os = "android")))]
fn get_instance_extensions() -> Vec<*const i8> {
    vec![Surface::name().as_ptr(),
         XlibSurface::name().as_ptr(),
         DebugReport::name().as_ptr()]
}

pub struct Renderer {
    pub instance: Arc<Instance>,
    render_target: RenderTarget,
    pub device: Arc<Device>,
    debug_report_loader: DebugReport,
    debug_call_back: vk::DebugReportCallbackEXT,

    pool: Pool,
    frame_buffers: Vec<vk::Framebuffer>,
    render_pass: vk::RenderPass,
    g_buffer: GBuffer,

    present_complete_semaphore: vk::Semaphore,
    rendering_complete_semaphore: vk::Semaphore,
    offscreen_semaphore: vk::Semaphore,
    mesh: Mesh,
    shader: Shader,
}

impl Renderer {
    pub fn init(engine_name: &str, app_name: &str, window: &winit::Window) -> Renderer { unsafe {
        let instance = Arc::new(Instance::init(engine_name, app_name));

        let debug_info = vk::DebugReportCallbackCreateInfoEXT {
            s_type: vk::StructureType::DebugReportCallbackCreateInfoExt,
            p_next: ptr::null(),
            flags: vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT |
                vk::DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT,
            pfn_callback: vulkan_debug_callback,
            p_user_data: ptr::null_mut(),
        };
        let debug_report_loader = DebugReport::new(&instance.entry, &instance.handle)
            .expect("Unable to load debug report");
        let debug_call_back =
            debug_report_loader.create_debug_report_callback_ext(&debug_info, None)
                .unwrap();

        let (render_target, device) =
            RenderTarget::create_render_target_and_device(instance.clone(), window);

        let pool = Pool::init(device.clone(), render_target.swap_chain.image_count);

        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SemaphoreCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
        };

        let renderpass_attachments = vec![
            vk::AttachmentDescription {
                format: render_target.capabilities.format.format,
                flags: vk::AttachmentDescriptionFlags::empty(),
                samples: vk::SAMPLE_COUNT_1_BIT,
                load_op: vk::AttachmentLoadOp::Clear,
                store_op: vk::AttachmentStoreOp::Store,
                stencil_load_op: vk::AttachmentLoadOp::DontCare,
                stencil_store_op: vk::AttachmentStoreOp::DontCare,
                initial_layout: vk::ImageLayout::Undefined,
                final_layout: vk::ImageLayout::PresentSrcKhr,
            },
            vk::AttachmentDescription {
                format: vk::Format::D16Unorm,
                flags: vk::AttachmentDescriptionFlags::empty(),
                samples: vk::SAMPLE_COUNT_1_BIT,
                load_op: vk::AttachmentLoadOp::Clear,
                store_op: vk::AttachmentStoreOp::Store,
                stencil_load_op: vk::AttachmentLoadOp::DontCare,
                stencil_store_op: vk::AttachmentStoreOp::DontCare,
                initial_layout: vk::ImageLayout::Undefined,
                final_layout: vk::ImageLayout::DepthStencilAttachmentOptimal,
            },
        ];

        let color_attachments_ref = vec![
            vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::ColorAttachmentOptimal}];

        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DepthStencilAttachmentOptimal,
        };
        let subpass = vk::SubpassDescription {
            color_attachment_count: color_attachments_ref.len() as u32,
            p_color_attachments: color_attachments_ref.as_ptr(),
            p_depth_stencil_attachment: &depth_attachment_ref,
            flags: Default::default(),
            pipeline_bind_point: vk::PipelineBindPoint::Graphics,
            input_attachment_count: 0,
            p_input_attachments: ptr::null(),
            p_resolve_attachments: ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: ptr::null(),
        };

        let dependencies = [
            vk::SubpassDependency {
                dependency_flags: vk::DEPENDENCY_BY_REGION_BIT,
                src_subpass: vk::VK_SUBPASS_EXTERNAL,
                dst_subpass: Default::default(),
                src_stage_mask: vk::PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT,
                src_access_mask: vk::ACCESS_MEMORY_READ_BIT,
                dst_stage_mask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
                dst_access_mask: vk::ACCESS_COLOR_ATTACHMENT_READ_BIT |
                    vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
            },
            vk::SubpassDependency {
                dependency_flags: vk::DEPENDENCY_BY_REGION_BIT,
                src_subpass: Default::default(),
                dst_subpass: vk::VK_SUBPASS_EXTERNAL,
                src_stage_mask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
                src_access_mask: Default::default(),
                dst_access_mask: vk::ACCESS_COLOR_ATTACHMENT_READ_BIT |
                    vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
                dst_stage_mask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
            }];
        let render_pass_create_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RenderPassCreateInfo,
            flags: Default::default(),
            p_next: ptr::null(),
            attachment_count: renderpass_attachments.len() as u32,
            p_attachments: renderpass_attachments.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: dependencies.len() as u32,
            p_dependencies: dependencies.as_ptr(),
        };
        let render_pass = device.create_render_pass(&render_pass_create_info, None).unwrap();

        let g_buffer = GBuffer::create_g_buffer(device.clone(), render_target.capabilities.resolution.clone(), &render_pass, pool.setup_command_buffer);
        let diffuse_texture = Texture::init(device.clone(), "assets/textures/MarbleGreen_COLOR.tga");
        let spec_texture = Texture::init(device.clone(), "assets/textures/MarbleGreen_NRM.tga");
        let mesh = Mesh::new(device.clone(), "assets/mesh/armour.obj", pool.setup_command_buffer);
        /*
        record_submit_commandbuffer(&device,
                                    pool.setup_command_buffer,
                                    &[vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT],
                                    &[],
                                    &[],
                                    |texture_command_buffer| {
                                        g_buffer.depth.transfer_data(texture_command_buffer);
                                        //diffuse_texture.load_texture(texture_command_buffer);
                                        //spec_texture.load_texture(texture_command_buffer);
                                    });
        */
        let frame_buffers: Vec<vk::Framebuffer> = render_target.swap_chain.image_views
            .iter()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view, g_buffer.depth.descriptor.image_view.clone()];
                let frame_buffer_create_info = vk::FramebufferCreateInfo {
                    s_type: vk::StructureType::FramebufferCreateInfo,
                    p_next: ptr::null(),
                    flags: Default::default(),
                    render_pass,
                    attachment_count: framebuffer_attachments.len() as u32,
                    p_attachments: framebuffer_attachments.as_ptr(),
                    width: render_target.capabilities.resolution.width,
                    height: render_target.capabilities.resolution.height,
                    layers: 1,
                };
                device.create_framebuffer(&frame_buffer_create_info, None).unwrap()
            })
            .collect();

        let present_complete_semaphore = device.create_semaphore(
            &semaphore_create_info, None).unwrap();
        let rendering_complete_semaphore = device.create_semaphore(
            &semaphore_create_info, None).unwrap();
        let offscreen_semaphore = device.create_semaphore(
            &semaphore_create_info, None).unwrap();


        let arc_d_texture = Arc::new(diffuse_texture);
        let arc_s_texture = Arc::new(spec_texture);
        let camera = Camera::new(Transform::from_position(Vector3::new(0.0, 0.0, 1.0)), 90.0);

        let mats: Vec<MVP> = (0..3).map(|i: i64| {
            let x = (i as f32) * 2.5;
                MVP::from_transform(&Transform::new(Vector3::new(x - 1.5, -1.0, -0.5 - (i as f32)), SMRotation::default(), Vector3::new(0.75, 0.75, 0.75)),
                                    &camera,
                                    render_target.capabilities.resolution.width, render_target.capabilities.resolution.height)
        }).collect::<Vec<MVP>>();

        let uniform_buffer = DynamicUniformBuffer::init(
            device.clone(),mats);

        let uniforms = vec![
            /*
            UniformDescriptor {
                data: arc_d_texture,
                stage: vk::SHADER_STAGE_FRAGMENT_BIT,
                binding: 1,
                set: 0,
            },
            UniformDescriptor {
                data: arc_s_texture,
                stage: vk::SHADER_STAGE_FRAGMENT_BIT,
                binding: 2,
                set: 0,
            },
            */
            UniformDescriptor {
                data: Arc::new(uniform_buffer),
                stage: vk::SHADER_STAGE_VERTEX_BIT,
                binding: 0,
                set: 0,
            }];
        let shader = Shader::from_file(device.clone(),
                                       &render_target.capabilities.resolution,
                                       &g_buffer.deferred_render_pass,
                                       "assets/shaders/texture.frag", "assets/shaders/texture.vert",
                                       true,
                                       uniforms);

        g_buffer.build_deferred_command_buffer(&pool.draw_command_buffer, &frame_buffers, &render_pass);
        g_buffer.build_scene_command_buffer(&pool, &mesh, &shader);
        Renderer{
            instance,
            device,
            render_target,
            debug_report_loader,
            debug_call_back,
            pool,
            frame_buffers,
            render_pass,
            g_buffer,
            present_complete_semaphore,
            rendering_complete_semaphore,
            offscreen_semaphore,
            mesh,
            shader
        }
    }}

    pub fn get_device(&self) -> &ash::Device<V1_0> {
        &self.device.handle
    }

    pub fn render(&self) { unsafe {

        let current_buffer = self.render_target.next_image(self.present_complete_semaphore);

        // off screen
        let mut submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SubmitInfo,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: &self.present_complete_semaphore,
            p_wait_dst_stage_mask: &vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
            command_buffer_count: 1,
            p_command_buffers: &self.pool.off_screen_command_buffer,
            signal_semaphore_count: 1,
            p_signal_semaphores: &self.offscreen_semaphore,
        };

        self.device.queue_submit(self.device.queue, &[submit_info.clone()], vk::Fence::null())
            .expect("offscreen submit failed");


        submit_info.p_wait_semaphores = &self.offscreen_semaphore;
        submit_info.p_signal_semaphores = &self.rendering_complete_semaphore;
        submit_info.p_command_buffers = &self.pool.draw_command_buffer[current_buffer as usize];
        self.device.queue_submit(self.device.queue, &[submit_info.clone()], vk::Fence::null())
            .expect("deferred submit failed");

        self.render_target.present(&self.rendering_complete_semaphore, current_buffer);
        self.device.queue_wait();

    }}
}

impl Drop for Renderer {
    fn drop(&mut self) { unsafe {
        self.device.device_wait_idle().unwrap();
        self.device.destroy_semaphore(self.present_complete_semaphore, None);
        self.device.destroy_semaphore(self.rendering_complete_semaphore, None);
        self.device.destroy_semaphore(self.offscreen_semaphore, None);
        self.debug_report_loader.destroy_debug_report_callback_ext(self.debug_call_back, None);
        for framebuffer in self.frame_buffers.clone() {
            self.device.destroy_framebuffer(framebuffer, None);
        }
        self.device.destroy_render_pass(self.render_pass, None);
    }}
}

unsafe extern "system" fn vulkan_debug_callback(_: vk::DebugReportFlagsEXT,
                                                _: vk::DebugReportObjectTypeEXT,
                                                _: u64,
                                                _: usize,
                                                _: i32,
                                                _: *const i8,
                                                p_message: *const i8,
                                                _: *mut libc::c_void)
                                                -> u32 {
    println!("{:?}", CStr::from_ptr(p_message));
    1
}

fn resize_callback(width: u32, height: u32) {
    println!("Window resized to {}x{}", width, height);
}
