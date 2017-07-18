use image;
use ash::vk;
pub use ash::version::{V1_0, InstanceV1_0, DeviceV1_0, EntryV1_0};
use ash::util::*;

use std::mem;
use std::mem::align_of;
use std::ptr;
use std::sync::Arc;
use std::path::Path;

use engine::renderer::memory::*;
use engine::renderer::device::Device;
use engine::renderer::shader::uniform::Uniform;

pub struct Texture {
    image_buffer_memory: vk::DeviceMemory,
    image_buffer: vk::Buffer,
    pub texture_image: Arc<Image>,
    descriptor: vk::DescriptorImageInfo,
    device: Arc<Device>
}

impl Texture {
    pub fn init<P: AsRef<Path>>(device: Arc<Device>, path: P) -> Texture { unsafe {
        let image = image::open(path).unwrap().to_rgba();
        let image_dimensions = image.dimensions();
        let image_data = image.into_raw();
        let image_buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BufferCreateInfo,
            p_next: ptr::null(),
            flags: vk::BufferCreateFlags::empty(),
            size: (mem::size_of::<u8>() * image_data.len()) as u64,
            usage: vk::BUFFER_USAGE_TRANSFER_SRC_BIT,
            sharing_mode: vk::SharingMode::Exclusive,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
        };
        let image_buffer = device.create_buffer(&image_buffer_info, None).unwrap();
        let image_buffer_memory_req = device.get_buffer_memory_requirements(image_buffer);
        let image_buffer_memory_index = find_memorytype_index(&image_buffer_memory_req,
                                                              &device.memory_properties,
                                                              vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT)
            .expect("Unable to find suitable memorytype for the vertex buffer.");

        let image_buffer_allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MemoryAllocateInfo,
            p_next: ptr::null(),
            allocation_size: image_buffer_memory_req.size,
            memory_type_index: image_buffer_memory_index,
        };
        let image_buffer_memory =
            device.allocate_memory(&image_buffer_allocate_info, None).unwrap();
        let image_buffer_ptr = device
            .map_memory(image_buffer_memory,
                              0,
                              image_buffer_info.size,
                              vk::MemoryMapFlags::empty())
            .unwrap();
        let mut image_buffer_slice = Align::new(image_buffer_ptr, align_of::<u8>() as u64, image_buffer_info.size);
        image_buffer_slice.copy_from_slice(&image_data);
        device.unmap_memory(image_buffer_memory);
        device.bind_buffer_memory(image_buffer, image_buffer_memory, 0).unwrap();

        let extent = vk::Extent2D { width: image_dimensions.0, height: image_dimensions.1};

        let texture_image = Arc::new(Image::create_sample(device.clone(),
                                        extent,
                                        vk::Format::R8g8b8a8Unorm,
                                        Usage::Texture,
                                        Swizzle::RGBA));

        let sampler_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SamplerCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            mag_filter: vk::Filter::Linear,
            min_filter: vk::Filter::Linear,
            mipmap_mode: vk::SamplerMipmapMode::Linear,
            address_mode_u: vk::SamplerAddressMode::MirroredRepeat,
            address_mode_v: vk::SamplerAddressMode::MirroredRepeat,
            address_mode_w: vk::SamplerAddressMode::MirroredRepeat,
            mip_lod_bias: 0.0,
            min_lod: 0.0,
            max_lod: 0.0,
            anisotropy_enable: 0,
            max_anisotropy: 1.0,
            border_color: vk::BorderColor::FloatOpaqueWhite,
            compare_enable: 0,
            compare_op: vk::CompareOp::Never,
            unnormalized_coordinates: 0,
        };

        let sampler = device.create_sampler(&sampler_info, None).unwrap();

        Texture {
            image_buffer_memory: image_buffer_memory,
            image_buffer: image_buffer,
            texture_image: texture_image.clone(),
            descriptor: vk::DescriptorImageInfo {
                image_layout: vk::ImageLayout::General,
                image_view: texture_image.view,
                sampler: sampler,
            },
            device: device.clone()
        } }
    }

    pub fn load_texture(&self, texture_command_buffer: vk::CommandBuffer) { unsafe {
        self.texture_image.transfer_data(texture_command_buffer);

        let buffer_copy_regions = [vk::BufferImageCopy {
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::IMAGE_ASPECT_COLOR_BIT,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_extent: vk::Extent3D {
                width: self.texture_image.dimensions.width,
                height: self.texture_image.dimensions.height,
                depth: 1,
            },
            buffer_offset: 0,
            // FIX ME
            buffer_image_height: 0,
            buffer_row_length: 0,
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
        }];
        self.device.cmd_copy_buffer_to_image(texture_command_buffer,
                                        self.image_buffer,
                                        self.texture_image.image,
                                        vk::ImageLayout::TransferDstOptimal,
                                        &buffer_copy_regions);
        let texture_barrier_end = vk::ImageMemoryBarrier {
            s_type: vk::StructureType::ImageMemoryBarrier,
            p_next: ptr::null(),
            src_access_mask: vk::ACCESS_TRANSFER_WRITE_BIT,
            dst_access_mask: vk::ACCESS_SHADER_READ_BIT,
            old_layout: vk::ImageLayout::TransferDstOptimal,
            new_layout: vk::ImageLayout::ShaderReadOnlyOptimal,
            src_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
            image: self.texture_image.image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::IMAGE_ASPECT_COLOR_BIT,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        };
        self.device.cmd_pipeline_barrier(texture_command_buffer,
                                    vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT,
                                    vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT,
                                    vk::DependencyFlags::empty(),
                                    &[],
                                    &[],
                                    &[texture_barrier_end]);
    }}
}

impl Drop for Texture {
    fn drop(&mut self) { unsafe {
        self.device.free_memory(self.image_buffer_memory, None);
        self.device.destroy_buffer(self.image_buffer, None);
        self.device.destroy_sampler(self.descriptor.sampler, None);
    }}
}

impl Uniform for Texture {
    fn get_descriptor_type(&self) -> vk::DescriptorType {
        vk::DescriptorType::CombinedImageSampler
    }
    fn image_info(&self) -> *const vk::DescriptorImageInfo {
        &self.descriptor
    }
}

pub enum Swizzle {
    Identity,
    RGBA
}

#[derive(Clone)]
pub enum Usage {
    Attachment,
    Texture,
    Depth
}

#[derive(Clone)]
pub struct Sample {
    pub image: Arc<Image>,
    descriptor: vk::DescriptorImageInfo,
}

impl Sample {
    pub fn from_sample(device: Arc<Device>,
                       extent: vk::Extent2D,
                       format: vk::Format,
                       usage: Usage,
                       swizzle: Swizzle,
                       sampler: vk::Sampler) -> Sample {
        let image = Image::create_sample(device, extent, format, usage, swizzle);
        Sample {
            image: Arc::new(image.clone()),
            descriptor: vk::DescriptorImageInfo {
                image_layout: vk::ImageLayout::ShaderReadOnlyOptimal,
                image_view: image.view,
                sampler: sampler,
            }
        }
    }
    pub fn transfer_data(&self, command_buffer: vk::CommandBuffer) {
        self.image.transfer_data(command_buffer);
    }
}

impl Uniform for Sample {
    fn get_descriptor_type(&self) -> vk::DescriptorType {
        vk::DescriptorType::CombinedImageSampler
    }
    fn image_info(&self) -> *const vk::DescriptorImageInfo {
        &self.descriptor
    }
}

#[derive(Clone)]
pub struct Image {
    device: Arc<Device>,
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub memory: vk::DeviceMemory,
    pub dimensions: vk::Extent2D,
    pub usage: Usage,
    pub format: vk::Format,

}

impl Image {
    pub fn init(device: Arc<Device>,
                extent: vk::Extent2D,
                format: vk::Format,
                usage: Usage,
                swizzle: Swizzle) -> Image {

        let create_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::ImageCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            image_type: vk::ImageType::Type2d,
            format: format,
            extent: vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SAMPLE_COUNT_1_BIT,
            tiling: vk::ImageTiling::Optimal,
            usage: match usage {
                Usage::Depth => vk::IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT,
                Usage::Texture => vk::IMAGE_USAGE_TRANSFER_DST_BIT ,
                Usage::Attachment => vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT},
            sharing_mode: vk::SharingMode::Exclusive,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
            initial_layout: vk::ImageLayout::Undefined,
        };
        Image::from_info(device, extent, format, usage, swizzle, create_info)
    }

    pub fn from_info(device: Arc<Device>,
                     extent: vk::Extent2D,
                     format: vk::Format,
                     usage: Usage,
                     swizzle: Swizzle,
                     create_info: vk::ImageCreateInfo) -> Image { unsafe {
        let depth_image = device.create_image(&create_info, None).unwrap();
        let depth_image_memory_req = device.get_image_memory_requirements(depth_image);
        let depth_image_memory_index =
            find_memorytype_index(&depth_image_memory_req,
                                  &device.memory_properties,
                                  vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT)
                .expect("Unable to find suitable memory index for depth image.");

        let depth_image_allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MemoryAllocateInfo,
            p_next: ptr::null(),
            allocation_size: depth_image_memory_req.size,
            memory_type_index: depth_image_memory_index,
        };
        let depth_image_memory = device.allocate_memory(&depth_image_allocate_info, None)
            .unwrap();
        device.bind_image_memory(depth_image, depth_image_memory, 0)
            .expect("Unable to bind depth image memory");

        let components = match swizzle {
            Swizzle::Identity => vk::ComponentMapping {
                r: vk::ComponentSwizzle::Identity,
                g: vk::ComponentSwizzle::Identity,
                b: vk::ComponentSwizzle::Identity,
                a: vk::ComponentSwizzle::Identity,
            },
            Swizzle::RGBA => vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            }
        };

        let aspect_mask = match usage {
            Usage::Depth => vk::IMAGE_ASPECT_DEPTH_BIT,
            Usage::Texture => vk::IMAGE_ASPECT_COLOR_BIT,
            _ => vk::IMAGE_ASPECT_COLOR_BIT,
        };

        let depth_image_view_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::ImageViewCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            view_type: vk::ImageViewType::Type2d,
            format: create_info.format,
            components: components,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: aspect_mask,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            image: depth_image,
        };
        let depth_image_view = device.create_image_view(&depth_image_view_info, None).unwrap();
        Image {
            device: device,
            image: depth_image,
            view: depth_image_view,
            memory: depth_image_memory,
            dimensions: extent,
            format: format,
            usage: usage,
        }
    }}

    pub fn create_sample(device: Arc<Device>,
                extent: vk::Extent2D,
                format: vk::Format,
                usage: Usage,
                swizzle: Swizzle) -> Image {
        let create_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::ImageCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            image_type: vk::ImageType::Type2d,
            format: format,
            extent: vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SAMPLE_COUNT_1_BIT,
            tiling: vk::ImageTiling::Optimal,
            usage: vk::IMAGE_USAGE_SAMPLED_BIT | match usage {
                Usage::Depth => vk::IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT,
                Usage::Texture => vk::IMAGE_USAGE_TRANSFER_DST_BIT ,
                Usage::Attachment => vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
            },
            sharing_mode: vk::SharingMode::Exclusive,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
            initial_layout: vk::ImageLayout::Undefined,
        };
        Image::from_info(device, extent, format, usage, swizzle, create_info)
    }

    pub fn transfer_data(&self, command_buffer: vk::CommandBuffer, ) { unsafe {

        let dst_access_mask = match self.usage {
            Usage::Depth => vk::ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT |
                vk::ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT,
            Usage::Texture => vk::ACCESS_TRANSFER_WRITE_BIT,
            _ => vk::ACCESS_TRANSFER_WRITE_BIT,
        };

        let new_layout = match self.usage {
            Usage::Depth => vk::ImageLayout::DepthStencilAttachmentOptimal,
            Usage::Texture => vk::ImageLayout::TransferDstOptimal,
            _ => vk::ImageLayout::TransferDstOptimal,
        };

        let aspect_mask = match self.usage {
            Usage::Depth => vk::IMAGE_ASPECT_DEPTH_BIT,
            Usage::Texture => vk::IMAGE_ASPECT_COLOR_BIT,
            _ => vk::IMAGE_ASPECT_COLOR_BIT,
        };

        let layout_transition_barrier = vk::ImageMemoryBarrier {
            s_type: vk::StructureType::ImageMemoryBarrier,
            p_next: ptr::null(),
            src_access_mask: Default::default(),
            dst_access_mask: dst_access_mask,
            old_layout: vk::ImageLayout::Undefined,
            new_layout: new_layout,
            src_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
            image: self.image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: aspect_mask,
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

impl Drop for Image {
    fn drop(&mut self) { unsafe {
        self.device.free_memory(self.memory, None);
        self.device.destroy_image_view(self.view, None);
        self.device.destroy_image(self.image, None);
    } }
}