use ash::vk;
pub use ash::version::{V1_0, InstanceV1_0, DeviceV1_0, EntryV1_0};

use renderer::device::Device;
use renderer::memory::*;
use renderer::shader::uniform::Uniform;
use renderer::shader::UniformDescriptor;
use cgmath::{Vector3, Vector4};

use std::ptr;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct Light {
    pub position: Vector4<f32>,
    pub color: Vector3<f32>,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Lights {
    pub lights: [Light; 3],
    pub view_pos: Vector4<f32>,
}

pub struct RenderPass {
    pub resolution: vk::Extent2D,
    pub colour_attachments: Vec<Attachment>,
    pub depth: Attachment,
    frame_buffers: Vec<vk::Framebuffer>,
    memory: vk::DeviceMemory,
    device: Arc<Device>,
    sampler: vk::Sampler,
    pub render_pass: vk::RenderPass,
}

impl RenderPass {
    pub fn new(device: Arc<Device>,
               resolution: vk::Extent2D,
               colour_req: Vec<(vk::Format, vk::ImageUsageFlags, vk::ImageLayout)>,
               depth_req: (vk::Format, vk::ImageUsageFlags, vk::ImageLayout),
               frame_buffer_image_views: Option<&Vec<vk::ImageView>>) -> Self { unsafe {
        if colour_req.len() == 0 {
            panic!("RenderTarget was not given any colour layers")
        }

        let sampler_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SamplerCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            mag_filter: vk::Filter::Nearest,
            min_filter: vk::Filter::Nearest,
            mipmap_mode: vk::SamplerMipmapMode::Linear,
            address_mode_u: vk::SamplerAddressMode::ClampToEdge,
            address_mode_v: vk::SamplerAddressMode::ClampToEdge,
            address_mode_w: vk::SamplerAddressMode::ClampToEdge,
            mip_lod_bias: 0.0,
            min_lod: 0.0,
            max_lod: 1.0,
            anisotropy_enable: 0,
            max_anisotropy: 1.0,
            border_color: vk::BorderColor::FloatOpaqueWhite,
            compare_enable: 0,
            compare_op: vk::CompareOp::Never,
            unnormalized_coordinates: 0,
        };

        let sampler = device.create_sampler(&sampler_info, None).unwrap();
        let mut req = colour_req.clone();
        req.push(depth_req);
        let (mut attachments, memory) = Attachment::create_attachments_with_layout(
            device.clone(), resolution.clone(), sampler.clone(),
            &req
        );

        let renderpass_attachments: Vec<vk::AttachmentDescription> = req.iter().map(|&(format, _, final_layout)| {
            vk::AttachmentDescription {
                format,
                flags: vk::AttachmentDescriptionFlags::empty(),
                samples: vk::SAMPLE_COUNT_1_BIT,
                load_op: vk::AttachmentLoadOp::Clear,
                store_op: vk::AttachmentStoreOp::Store,
                stencil_load_op: vk::AttachmentLoadOp::DontCare,
                stencil_store_op: vk::AttachmentStoreOp::DontCare,
                initial_layout: vk::ImageLayout::Undefined,
                final_layout,
            }
        }).collect();

        let mut color_attachments_ref = Vec::new();
        for i in 0..colour_req.len() {
            color_attachments_ref.push(vk::AttachmentReference {
                attachment: i as u32,
                layout: vk::ImageLayout::ColorAttachmentOptimal,
            });
        }

        let depth_attachment_ref = vk::AttachmentReference {
            attachment: color_attachments_ref.len() as u32,
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
        let deferred_render_pass_create_info = vk::RenderPassCreateInfo {
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
        let render_pass = device.create_render_pass(&deferred_render_pass_create_info, None).unwrap();

        let (frame_buffers, depth): (Vec<vk::Framebuffer>, Attachment) = match frame_buffer_image_views {
            Some(f) =>{
                let depth = attachments.pop().unwrap();
                (f
                    .iter()
                    .map(|&present_image_view| {
                        let framebuffer_attachments = [present_image_view, depth.descriptor.image_view.clone()];
                        let frame_buffer_create_info = vk::FramebufferCreateInfo {
                            s_type: vk::StructureType::FramebufferCreateInfo,
                            p_next: ptr::null(),
                            flags: Default::default(),
                            render_pass,
                            attachment_count: framebuffer_attachments.len() as u32,
                            p_attachments: framebuffer_attachments.as_ptr(),
                            width: resolution.width,
                            height: resolution.height,
                            layers: 1,
                        };
                        device.create_framebuffer(&frame_buffer_create_info, None).unwrap()
                    })
                    .collect(), depth)
            },
            None => {
                let attachments_views: Vec<vk::ImageView> = attachments.iter().map(|a| {
                    a.descriptor.image_view
                }).collect();

                let frame_buffer_create_info = vk::FramebufferCreateInfo {
                    s_type: vk::StructureType::FramebufferCreateInfo,
                    p_next: ptr::null(),
                    flags: Default::default(),
                    render_pass,
                    attachment_count: attachments_views.len() as u32,
                    p_attachments: attachments_views.as_ptr(),
                    width: resolution.width,
                    height: resolution.height,
                    layers: 1,
                };
                (vec![device.create_framebuffer(&frame_buffer_create_info, None).unwrap()], attachments.pop().unwrap())
            }
        };

        Self {resolution, sampler, device: device.clone(), memory, render_pass, frame_buffers, depth, colour_attachments: attachments}
    }}

    pub unsafe fn record_commands<F: Fn(vk::CommandBuffer)>(&self, commands: &Vec<vk::CommandBuffer>, f: &F) {
        let mut clear_values: Vec<vk::ClearValue> = self.colour_attachments.iter().map(|_x|{
            vk::ClearValue::new_color(vk::ClearColorValue::new_float32([0.0, 0.0, 0.0, 0.0]))
        }).collect();
        clear_values.push(
            vk::ClearValue::new_depth_stencil(vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            }));

        let render_pass_begin_infos: Vec<vk::RenderPassBeginInfo> = self.frame_buffers.iter().map(|frame_buffer|{
            vk::RenderPassBeginInfo {
                s_type: vk::StructureType::RenderPassBeginInfo,
                p_next: ptr::null(),
                render_pass: self.render_pass,
                framebuffer: frame_buffer.clone(),
                render_area: vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.resolution.clone(),
                },
                clear_value_count: clear_values.len() as u32,
                p_clear_values: clear_values.as_ptr(),
            }}).collect();

        for i in 0..commands.len() {
            let command_buffer_begin_info = vk::CommandBufferBeginInfo {
                s_type: vk::StructureType::CommandBufferBeginInfo,
                p_next: ptr::null(),
                p_inheritance_info: ptr::null(),
                flags: vk::COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT
            };
            self.device.begin_command_buffer(commands[i], &command_buffer_begin_info).expect("Begin commandbuffer");
            self.device.cmd_begin_render_pass(commands[i], &render_pass_begin_infos[i], vk::SubpassContents::Inline);
            f(commands[i]);
            self.device.end_command_buffer(commands[i]).expect("End commandbuffer");
        }
    }
    pub fn attachment_to_uniform(&self, set: u32, offset: u32) -> Vec<UniformDescriptor> {
        self.colour_attachments.iter().enumerate().map(|(i, attachment)| {
            UniformDescriptor {
                data: Arc::new(attachment.clone()),
                stage: vk::SHADER_STAGE_FRAGMENT_BIT,
                binding: (i as u32) + offset,
                set,
            }
        }).collect()
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) { unsafe {
        for frame_buffer in &self.frame_buffers {
            self.device.destroy_framebuffer(frame_buffer.clone(), None);
        }
        self.device.destroy_render_pass(self.render_pass, None);
        self.device.destroy_sampler(self.sampler, None);

        for attachment in &self.colour_attachments {
            self.device.destroy_image_view(attachment.descriptor.image_view, None);
            self.device.destroy_image(attachment.image, None);
        }

        self.device.destroy_image_view(self.depth.descriptor.image_view, None);
        self.device.destroy_image(self.depth.image, None);
        self.device.free_memory(self.memory, None);
    }}
}

#[derive(Clone)]
pub struct Attachment {
    image: vk::Image,
    format: vk::Format,
    device: Arc<Device>,
    usage: vk::ImageUsageFlags,
    pub descriptor: vk::DescriptorImageInfo,
}

impl Attachment {
    pub fn create_attachments(device: Arc<Device>, extent: vk::Extent2D, sampler: vk::Sampler, req: &Vec<(vk::Format, vk::ImageUsageFlags)>) -> (Vec<Attachment>, vk::DeviceMemory) { unsafe {
        let images: Vec<(vk::Image, vk::Format, vk::MemoryRequirements, vk::ImageAspectFlags, vk::ImageUsageFlags)> = req.iter().map(|&(format, usage)| {
            let aspect_mask = if usage == vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT {
                vk::IMAGE_ASPECT_COLOR_BIT
            } else {
                vk::IMAGE_ASPECT_DEPTH_BIT
            };

            let image_info = vk::ImageCreateInfo {
                s_type: vk::StructureType::ImageCreateInfo,
                p_next: ptr::null(),
                flags: Default::default(),
                image_type: vk::ImageType::Type2d,
                format,
                extent: vk::Extent3D {
                    width: extent.width,
                    height: extent.height,
                    depth: 1,
                },
                mip_levels: 1,
                array_layers: 1,
                samples: vk::SAMPLE_COUNT_1_BIT,
                tiling: vk::ImageTiling::Optimal,
                usage: usage | vk::IMAGE_USAGE_SAMPLED_BIT,
                sharing_mode: vk::SharingMode::Exclusive,
                queue_family_index_count: 0,
                p_queue_family_indices: ptr::null(),
                initial_layout: vk::ImageLayout::Undefined,
            };

            let image = device.create_image(&image_info, None).unwrap();
            let req = device.get_image_memory_requirements(image);

            (image, format, req, aspect_mask, usage)
        }).collect();

        let sizes: Vec<u64> =
            images.iter().map(|x| x.2.size.clone()).collect();

        let (_, _, mem_req, _, _) = images[0].clone();

        let image_memory_index = find_memorytype_index(&mem_req,
                                                       &device.memory_properties,
                                                       vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT).unwrap();
        let mem_alloc = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MemoryAllocateInfo,
            p_next: ptr::null(),
            allocation_size: sizes.iter().sum(),
            memory_type_index: image_memory_index,
        };

        let memory = device.allocate_memory(&mem_alloc, None).unwrap();

        for i in 0..images.len() {
            device.bind_image_memory(images[i].0, memory, ((0..i).fold(0, |sum, x| sum + sizes[x]))).expect("Unable to bind depth image memory");
        }
        (images.into_iter().map(|(image, format, _, aspect_mask, usage)| {
            let view_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::ImageViewCreateInfo,
                p_next: ptr::null(),
                flags: Default::default(),
                view_type: vk::ImageViewType::Type2d,
                format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::Identity,
                    g: vk::ComponentSwizzle::Identity,
                    b: vk::ComponentSwizzle::Identity,
                    a: vk::ComponentSwizzle::Identity,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                image,
            };

            let view = device.create_image_view(&view_info, None).unwrap();

            Attachment {
                image,
                format,
                usage,
                descriptor: vk::DescriptorImageInfo {
                    image_layout: vk::ImageLayout::ShaderReadOnlyOptimal,
                    image_view: view,
                    sampler,
                },
                device: device.clone()
            }
        }).collect(), memory)
    }}

    pub fn create_attachments_with_layout(device: Arc<Device>, extent: vk::Extent2D, sampler: vk::Sampler, req: &Vec<(vk::Format, vk::ImageUsageFlags, vk::ImageLayout)>) -> (Vec<Attachment>, vk::DeviceMemory) { unsafe {
        let images: Vec<(vk::Image, vk::Format, vk::MemoryRequirements, vk::ImageAspectFlags, vk::ImageUsageFlags)> = req.iter().map(|&(format, usage, _)| {
            let aspect_mask = if usage == vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT {
                vk::IMAGE_ASPECT_COLOR_BIT
            } else {
                vk::IMAGE_ASPECT_DEPTH_BIT
            };

            let image_info = vk::ImageCreateInfo {
                s_type: vk::StructureType::ImageCreateInfo,
                p_next: ptr::null(),
                flags: Default::default(),
                image_type: vk::ImageType::Type2d,
                format,
                extent: vk::Extent3D {
                    width: extent.width,
                    height: extent.height,
                    depth: 1,
                },
                mip_levels: 1,
                array_layers: 1,
                samples: vk::SAMPLE_COUNT_1_BIT,
                tiling: vk::ImageTiling::Optimal,
                usage: usage | vk::IMAGE_USAGE_SAMPLED_BIT,
                sharing_mode: vk::SharingMode::Exclusive,
                queue_family_index_count: 0,
                p_queue_family_indices: ptr::null(),
                initial_layout: vk::ImageLayout::Undefined,
            };

            let image = device.create_image(&image_info, None).unwrap();
            let req = device.get_image_memory_requirements(image);

            (image, format, req, aspect_mask, usage)
        }).collect();

        let sizes: Vec<u64> =
            images.iter().map(|x| x.2.size.clone()).collect();

        let (_, _, mem_req, _, _) = images[0].clone();

        let image_memory_index = find_memorytype_index(&mem_req,
                                                       &device.memory_properties,
                                                       vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT).unwrap();
        let mem_alloc = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MemoryAllocateInfo,
            p_next: ptr::null(),
            allocation_size: sizes.iter().sum(),
            memory_type_index: image_memory_index,
        };

        let memory = device.allocate_memory(&mem_alloc, None).unwrap();

        for i in 0..images.len() {
            device.bind_image_memory(images[i].0, memory, ((0..i).fold(0, |sum, x| sum + sizes[x]))).expect("Unable to bind depth image memory");
        }
        (images.into_iter().map(|(image, format, _, aspect_mask, usage)| {
            let view_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::ImageViewCreateInfo,
                p_next: ptr::null(),
                flags: Default::default(),
                view_type: vk::ImageViewType::Type2d,
                format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::Identity,
                    g: vk::ComponentSwizzle::Identity,
                    b: vk::ComponentSwizzle::Identity,
                    a: vk::ComponentSwizzle::Identity,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                image,
            };

            let view = device.create_image_view(&view_info, None).unwrap();

            Attachment {
                image,
                format,
                usage,
                descriptor: vk::DescriptorImageInfo {
                    image_layout: vk::ImageLayout::ShaderReadOnlyOptimal,
                    image_view: view,
                    sampler,
                },
                device: device.clone()
            }
        }).collect(), memory)
    }}

    pub fn transfer_data(&self, command_buffer: vk::CommandBuffer) { unsafe {
        let dst_access_mask =
            if self.usage == vk::IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT {
                vk::ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT | vk::ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT
            } else {
                vk::ACCESS_TRANSFER_WRITE_BIT
            };

        let new_layout =
            if self.usage == vk::IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT {
                vk::ImageLayout::DepthStencilAttachmentOptimal
            } else {
                vk::ImageLayout::TransferDstOptimal
            };

        let aspect_mask =
            if self.usage == vk::IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT {
            vk::IMAGE_ASPECT_DEPTH_BIT
        } else {
            vk::IMAGE_ASPECT_COLOR_BIT
        };

        let layout_transition_barrier = vk::ImageMemoryBarrier {
            s_type: vk::StructureType::ImageMemoryBarrier,
            p_next: ptr::null(),
            src_access_mask: Default::default(),
            dst_access_mask,
            old_layout: vk::ImageLayout::TransferDstOptimal,
            new_layout,
            src_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
            image: self.image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        };
        self.device.cmd_pipeline_barrier(command_buffer,
                                         vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT,
                                         vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT,
                                         vk::DependencyFlags::empty(),
                                         &[],
                                         &[],
                                         &[layout_transition_barrier]);
    }}
}

impl Uniform for Attachment {
    fn get_descriptor_type(&self) -> vk::DescriptorType {
        vk::DescriptorType::CombinedImageSampler
    }
    fn image_info(&self) -> *const vk::DescriptorImageInfo {
        &self.descriptor
    }
}
