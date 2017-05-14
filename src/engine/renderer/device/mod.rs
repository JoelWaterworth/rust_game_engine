use ash;
use ash::vk;
use ash::vk::*;
use std::default::Default;
use std::ptr;

use ash::version::{InstanceV1_0, DeviceV1_0, V1_0};
use ash::extensions::{Swapchain};
use std::ops::Drop;

use std::sync::Arc;

use engine::renderer::surface::RVSurface;
use engine::renderer;
use engine::renderer::memory::find_memorytype_index;

use std::u32;

pub struct Device {
    pub queue_family_index: u32,
    pub handle: ash::Device<V1_0>,
    pub queue: Queue,
    pub memory_properties: PhysicalDeviceMemoryProperties,
    pub device_properties: PhysicalDeviceProperties,
    instance: Arc<renderer::Instance>,
    p_device: PhysicalDevice,
}

impl Device {
    pub fn init(instance: Arc<renderer::Instance>, queue_family_index: u32, p_device: PhysicalDevice) -> Device { unsafe {
        let device_extension_names = get_device_extensions();
        let features =
            PhysicalDeviceFeatures { shader_clip_distance: 1, ..Default::default() };
        let priorities = [1.0];
        let queue_info = DeviceQueueCreateInfo {
            s_type: StructureType::DeviceQueueCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            queue_family_index: queue_family_index.clone() as u32,
            p_queue_priorities: priorities.as_ptr(),
            queue_count: priorities.len() as u32,
        };
        let device_create_info = DeviceCreateInfo {
            s_type: StructureType::DeviceCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            queue_create_info_count: 1,
            p_queue_create_infos: &queue_info,
            enabled_layer_count: 0, // device layers are deprecated
            pp_enabled_layer_names: ptr::null(),
            enabled_extension_count: device_extension_names.len() as u32,
            pp_enabled_extension_names: device_extension_names.as_ptr(),
            p_enabled_features: &features,
        };
        let device: ash::Device<V1_0> = instance.create_device(p_device, &device_create_info, None)
            .unwrap();
        let present_queue = device.get_device_queue(queue_family_index.clone() as u32, 0);

        let device_memory_properties = instance.get_physical_device_memory_properties(p_device);

        let device_properties = instance.get_physical_device_properties(p_device);

        Device{queue_family_index: queue_family_index, handle: device, queue: present_queue, instance: instance, memory_properties: device_memory_properties, device_properties: device_properties, p_device: p_device}
    } }

    pub fn allocate_suitable_memory(&self, buffer: vk::Buffer) -> vk::DeviceMemory { unsafe {
        let memory_req = self.get_buffer_memory_requirements(buffer);
        let memory_index = find_memorytype_index(&memory_req,
                                                              &self.memory_properties,
                                                              vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT)
            .expect("Unable to find suitable memorytype for the index buffer.");

        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MemoryAllocateInfo,
            p_next: ptr::null(),
            allocation_size: memory_req.size,
            memory_type_index: memory_index,
        };

        self.allocate_memory(&allocate_info, None).unwrap()
    }}

    pub fn get_memory_type(&self, memory_req: &vk::MemoryRequirements, properties: vk::MemoryPropertyFlags) -> Option<u32> {
        let mut memory_type_bits = memory_req.memory_type_bits;
        for (i, ref memory_type) in self.memory_properties.memory_types.iter().enumerate() {
            if (memory_req.memory_type_bits & 1) == 1 {
                if (memory_type.property_flags & properties) == properties {
                    return Some(i as u32);
                }
                else if memory_type.property_flags == properties {
                    return Some(i as u32);
                }
            }
            memory_type_bits = memory_type_bits >> 1;
        }
        None
    }

    pub fn queue_wait(&self) { unsafe {
        self.queue_wait_idle(self.queue).unwrap();
    }}
}

fn get_device_extensions() -> Vec<*const i8> {
    vec![Swapchain::name().as_ptr()]
}

pub fn get_usable_gpu(instance: &Arc<renderer::Instance>, surface: &RVSurface) -> (PhysicalDevice, u32) {
    let p_devices: Vec<PhysicalDevice> = instance.enumerate_physical_devices().expect("Physical device error");
    p_devices.iter()
        .map(|p_device| {
            instance.get_physical_device_queue_family_properties(*p_device)
                .iter()
                .enumerate()
                .filter_map(|(index, ref info)| {
                    let supports_graphic_and_surface =
                        info.queue_flags.subset(QUEUE_GRAPHICS_BIT) &&
                            surface.loader.get_physical_device_surface_support_khr(*p_device,
                                                                                           index as u32,
                                                                                           surface.handle);
                    match supports_graphic_and_surface {
                        true => Some((*p_device, index as u32)),
                        _ => None,
                    }
                })
                .nth(0)
        })
        .filter_map(|v| v)
        .nth(0)
        .expect("Couldn't find suitable device.")
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.destroy_device(None);
        }
    }
}

impl DeviceV1_0 for Device {
    fn handle(&self) -> types::Device{
        self.handle.handle()
    }
    fn fp_v1_0(&self) -> &DeviceFnV1_0{
        self.handle.fp_v1_0()
    }
}
