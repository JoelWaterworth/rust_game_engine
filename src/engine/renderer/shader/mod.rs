use ash::vk;
use std::default::Default;
use std::ptr;
use std::ffi::CString;
use std::mem;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use glsl_to_spirv::{compile, ShaderType};

//use ash::Instance;
//use ash::Device;
pub use ash::version::{V1_0, InstanceV1_0, DeviceV1_0, EntryV1_0};
use std::ops::Drop;

use std::sync::Arc;

use engine::renderer::device::Device;
use engine::renderer::mesh::Vertex;

use std::u32;

pub mod uniform;
use self::uniform::*;

pub struct UniformDescriptor {
    pub data: Arc<Uniform>,
    pub stage: vk::ShaderStageFlags,
    pub binding: u32,
    pub set: u32,
}

macro_rules! offset_of{
    ($base: path, $field: ident) => {
        {
            #[allow(unused_unsafe)]
            unsafe{
                let b: $base = mem::uninitialized();
                (&b.$field as *const _ as isize) - (&b as *const _ as isize)
            }
        }
    }
}

pub struct Shader {
    pub device: Arc<Device>,
    pub graphics_pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub scissors: Vec<vk::Rect2D>,
    pub viewports: Vec<vk::Viewport>,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    descriptor_set_layout: Vec<vk::DescriptorSetLayout>,
    descriptor_pool: vk::DescriptorPool,
    uniform_buffers: Vec<UniformDescriptor>,
}

impl Shader {
    #[allow(unused_must_use)]
    pub fn from_file<P: AsRef<Path>>(device: Arc<Device>,
                                     resolution: &vk::Extent2D,
                                     render_pass: &vk::RenderPass,
                                     frag_path: P, vertex_path: P,
                                     deferred: bool,
                                     uniforms: Vec<UniformDescriptor>) -> Shader {
        let mut frag_glsl_file = File::open(frag_path)
            .expect("Could not find texture.frag.");

        let mut frag_spv_string = String::new();
        frag_glsl_file.read_to_string(&mut frag_spv_string);
        let frag_spv_file = compile(frag_spv_string.as_str(), ShaderType::Fragment).unwrap();
        let frag_bytes: Vec<u8> = frag_spv_file.bytes().filter_map(|byte| byte.ok()).collect();

        let mut vertex_glsl_string = File::open(vertex_path)
            .expect("Could not find texture.frag.");

        let mut vertex_spv_string = String::new();
        vertex_glsl_string.read_to_string(&mut vertex_spv_string);
        let vertex_spv_file = compile(vertex_spv_string.as_str(), ShaderType::Vertex).unwrap();
        let vertex_bytes: Vec<u8> = vertex_spv_file.bytes().filter_map(|byte| byte.ok()).collect();

        Shader::from_spriv(device,
                           resolution,
                           render_pass,
                           frag_bytes,
                           vertex_bytes,
                            deferred,
                           uniforms)
    }

    pub fn from_spriv(device: Arc<Device>,
                      resolution: &vk::Extent2D,
                      render_pass: &vk::RenderPass,
                      frag_bytes: Vec<u8>, vertex_bytes: Vec<u8>,
                      deferred: bool,
                      uniforms: Vec<UniformDescriptor>) -> Shader { unsafe {
        let type_counts: Vec<vk::DescriptorPoolSize> = uniforms.iter()
            .map(| uniform | {
                vk::DescriptorPoolSize {
                    typ: uniform.data.get_descriptor_type(),
                    descriptor_count: 1,
                }
            }).collect();

        let descriptor_pool_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DescriptorPoolCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            max_sets: 1,
            pool_size_count: type_counts.len() as u32,
            p_pool_sizes: type_counts.as_ptr(),
        };

        let descriptor_pool = device.create_descriptor_pool(&descriptor_pool_info, None).unwrap();

        let layout_binding: Vec<vk::DescriptorSetLayoutBinding> =
            uniforms.iter().map(|x|{
                vk::DescriptorSetLayoutBinding {
                    binding: x.binding.clone(),
                    descriptor_type: x.data.get_descriptor_type(),
                    descriptor_count: 1,
                    stage_flags: x.stage.clone(),
                    p_immutable_samplers: ptr::null(),
                }
            }).collect();

        let descriptor_layout  = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DescriptorSetLayoutCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            binding_count: layout_binding.len() as u32,
            p_bindings: layout_binding.as_ptr(),
        };

        let descriptor_set_layout = vec![device.create_descriptor_set_layout(&descriptor_layout, None).unwrap()];

        let alloc_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DescriptorSetAllocateInfo,
            p_next: ptr::null(),
            descriptor_pool,
            descriptor_set_count: descriptor_set_layout.len() as u32,
            p_set_layouts: descriptor_set_layout.as_ptr(),
        };

        let descriptor_sets = device.allocate_descriptor_sets(&alloc_info).unwrap();

        let write_descriptor_sets: Vec<vk::WriteDescriptorSet> =
            uniforms.iter().map(|x|{
                vk::WriteDescriptorSet {
                    s_type: vk::StructureType::WriteDescriptorSet,
                    p_next: ptr::null(),
                    dst_set: descriptor_sets[0],
                    dst_binding: x.binding,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: x.data.get_descriptor_type(),
                    p_image_info: x.data.image_info(),
                    p_buffer_info: x.data.buffer_info(),
                    p_texel_buffer_view: x.data.texel_buffer_view(),
                }
            }).collect();

        device.update_descriptor_sets(&write_descriptor_sets, &[]);

        let vertex_shader_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::ShaderModuleCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            code_size: vertex_bytes.len(),
            p_code: vertex_bytes.as_ptr() as *const u32,
        };

        let frag_shader_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::ShaderModuleCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            code_size: frag_bytes.len(),
            p_code: frag_bytes.as_ptr() as *const u32,
        };
        let vertex_shader_module = device
            .create_shader_module(&vertex_shader_info, None)
            .expect("Vertex shader module error");

        let fragment_shader_module = device
            .create_shader_module(&frag_shader_info, None)
            .expect("Fragment shader module error");

        let shader_entry_name = CString::new("main").unwrap();
        let shader_stage_create_infos =
            [
                vk::PipelineShaderStageCreateInfo {
                    s_type: vk::StructureType::PipelineShaderStageCreateInfo,
                    p_next: ptr::null(),
                    flags: Default::default(),
                    module: vertex_shader_module,
                    p_name: shader_entry_name.as_ptr(),
                    p_specialization_info: ptr::null(),
                    stage: vk::SHADER_STAGE_VERTEX_BIT,
                },
                vk::PipelineShaderStageCreateInfo {
                    s_type: vk::StructureType::PipelineShaderStageCreateInfo,
                    p_next: ptr::null(),
                    flags: Default::default(),
                    module: fragment_shader_module,
                    p_name: shader_entry_name.as_ptr(),
                    p_specialization_info: ptr::null(),
                    stage: vk::SHADER_STAGE_FRAGMENT_BIT,
                }];
        let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::Vertex,
        }];
        let vertex_input_attribute_descriptions = [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32g32b32a32Sfloat,
                offset: offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32g32b32a32Sfloat,
                offset: offset_of!(Vertex, tangent) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: vk::Format::R32g32b32a32Sfloat,
                offset: offset_of!(Vertex, normal) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 3,
                binding: 0,
                format: vk::Format::R32g32b32a32Sfloat,
                offset: offset_of!(Vertex, uv) as u32,
            }];
        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PipelineVertexInputStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            vertex_attribute_description_count: vertex_input_attribute_descriptions.len() as u32,
            p_vertex_attribute_descriptions: vertex_input_attribute_descriptions.as_ptr(),
            vertex_binding_description_count: vertex_input_binding_descriptions.len() as u32,
            p_vertex_binding_descriptions: vertex_input_binding_descriptions.as_ptr(),
        };
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PipelineInputAssemblyStateCreateInfo,
            flags: Default::default(),
            p_next: ptr::null(),
            primitive_restart_enable: 0,
            topology: vk::PrimitiveTopology::TriangleList,
        };
        let viewports = vec![vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: resolution.width as f32,
            height: resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = vec![vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: resolution.clone(),
        }];
        let viewport_state_info = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PipelineViewportStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            scissor_count: scissors.len() as u32,
            p_scissors: scissors.as_ptr(),
            viewport_count: viewports.len() as u32,
            p_viewports: viewports.as_ptr(),
        };
        let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PipelineRasterizationStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            cull_mode: vk::CULL_MODE_NONE,
            depth_bias_clamp: 0.0,
            depth_bias_constant_factor: 0.0,
            depth_bias_enable: 0,
            depth_bias_slope_factor: 0.0,
            depth_clamp_enable: 0,
            front_face: vk::FrontFace::CounterClockwise,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::Fill,
            rasterizer_discard_enable: 0,
        };
        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PipelineMultisampleStateCreateInfo,
            flags: Default::default(),
            p_next: ptr::null(),
            rasterization_samples: vk::SAMPLE_COUNT_1_BIT,
            sample_shading_enable: 0,
            min_sample_shading: 0.0,
            p_sample_mask: ptr::null(),
            alpha_to_one_enable: 0,
            alpha_to_coverage_enable: 0,
        };
        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::Keep,
            pass_op: vk::StencilOp::Keep,
            depth_fail_op: vk::StencilOp::Keep,
            compare_op: vk::CompareOp::Always,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        };
        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PipelineDepthStencilStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LessOrEqual,
            depth_bounds_test_enable: 0,
            stencil_test_enable: 0,
            front: noop_stencil_state.clone(),
            back: noop_stencil_state.clone(),
            max_depth_bounds: 1.0,
            min_depth_bounds: 0.0,
        };
        let color_blend_attachment_states = if deferred { vec![vk::PipelineColorBlendAttachmentState {
                blend_enable: 0,
                src_color_blend_factor: vk::BlendFactor::SrcColor,
                dst_color_blend_factor:
                vk::BlendFactor::OneMinusDstColor,
                color_blend_op: vk::BlendOp::Add,
                src_alpha_blend_factor: vk::BlendFactor::Zero,
                dst_alpha_blend_factor: vk::BlendFactor::Zero,
                alpha_blend_op: vk::BlendOp::Add,
                color_write_mask: vk::ColorComponentFlags::all(),
            },
            vk::PipelineColorBlendAttachmentState {
                blend_enable: 0,
                src_color_blend_factor: vk::BlendFactor::SrcColor,
                dst_color_blend_factor:
                vk::BlendFactor::OneMinusDstColor,
                color_blend_op: vk::BlendOp::Add,
                src_alpha_blend_factor: vk::BlendFactor::Zero,
                dst_alpha_blend_factor: vk::BlendFactor::Zero,
                alpha_blend_op: vk::BlendOp::Add,
                color_write_mask: vk::ColorComponentFlags::all(),
            },
            vk::PipelineColorBlendAttachmentState {
                blend_enable: 0,
                src_color_blend_factor: vk::BlendFactor::SrcColor,
                dst_color_blend_factor:
                vk::BlendFactor::OneMinusDstColor,
                color_blend_op: vk::BlendOp::Add,
                src_alpha_blend_factor: vk::BlendFactor::Zero,
                dst_alpha_blend_factor: vk::BlendFactor::Zero,
                alpha_blend_op: vk::BlendOp::Add,
                color_write_mask: vk::ColorComponentFlags::all(),
            }

        ]} else {
            vec![vk::PipelineColorBlendAttachmentState {
                blend_enable: 0,
                src_color_blend_factor: vk::BlendFactor::SrcColor,
                dst_color_blend_factor:
                vk::BlendFactor::OneMinusDstColor,
                color_blend_op: vk::BlendOp::Add,
                src_alpha_blend_factor: vk::BlendFactor::Zero,
                dst_alpha_blend_factor: vk::BlendFactor::Zero,
                alpha_blend_op: vk::BlendOp::Add,
                color_write_mask: vk::ColorComponentFlags::all(),
            }]
        };

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PipelineColorBlendStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            logic_op_enable: 0,
            logic_op: vk::LogicOp::Clear,
            attachment_count: color_blend_attachment_states.len() as u32,
            p_attachments: color_blend_attachment_states.as_ptr(),
            blend_constants: [0.0, 0.0, 0.0, 0.0],
        };

        let dynamic_state = [vk::DynamicState::Viewport, vk::DynamicState::Scissor];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::StructureType::PipelineDynamicStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            dynamic_state_count: dynamic_state.len() as u32,
            p_dynamic_states: dynamic_state.as_ptr(),
        };

        let layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PipelineLayoutCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            set_layout_count: descriptor_set_layout.len() as u32,
            p_set_layouts: descriptor_set_layout.as_ptr(),
            push_constant_range_count: 0,
            p_push_constant_ranges: ptr::null(),
        };

        let pipeline_layout =
            device.create_pipeline_layout(&layout_create_info, None).unwrap();

        let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GraphicsPipelineCreateInfo,
            p_next: ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
            stage_count: shader_stage_create_infos.len() as u32,
            p_stages: shader_stage_create_infos.as_ptr(),
            p_vertex_input_state: &vertex_input_state_info,
            p_input_assembly_state: &vertex_input_assembly_state_info,
            p_tessellation_state: ptr::null(),
            p_viewport_state: &viewport_state_info,
            p_rasterization_state: &rasterization_info,
            p_multisample_state: &multisample_state_info,
            p_depth_stencil_state: &depth_state_info,
            p_color_blend_state: &color_blend_state,
            p_dynamic_state: &dynamic_state_info,
            layout: pipeline_layout,
            render_pass: render_pass.clone(),
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: 0,
        };
        let graphics_pipelines = device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[graphic_pipeline_info], None)
            .unwrap();

        device.destroy_shader_module(vertex_shader_module, None);
        device.destroy_shader_module(fragment_shader_module, None);

        Shader{device: device.clone()
            ,graphics_pipeline: graphics_pipelines[0],
            pipeline_layout: pipeline_layout,
            scissors: scissors,
            viewports: viewports,
            descriptor_sets: descriptor_sets,
            descriptor_set_layout: descriptor_set_layout,
            descriptor_pool: descriptor_pool,
            uniform_buffers: uniforms}
    } }
}

impl Drop for Shader {
    fn drop(&mut self) { unsafe {
        self.device.destroy_pipeline(self.graphics_pipeline, None);
        self.device.destroy_pipeline_layout(self.pipeline_layout, None);
        for &x in self.descriptor_set_layout.iter() {
            self.device.destroy_descriptor_set_layout(x , None);
        }
        self.device.destroy_descriptor_pool(self.descriptor_pool, None);
    }}
}