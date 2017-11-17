use ash::vk;
use std::default::Default;
use std::ptr;

//use ash::Instance;
//use ash::Device;
pub use ash::version::{V1_0, InstanceV1_0, DeviceV1_0, EntryV1_0};
use ash::extensions::{Swapchain, Surface, Win32Surface, XlibSurface};
use ash::vk::{uint32_t, SurfaceTransformFlagsKHR};
use std::ops::Drop;

use std::sync::Arc;

use engine::renderer::device::*;
use engine::renderer::Instance;

use winit;
use winit::Window;
#[cfg(windows)]
use user32;
#[cfg(windows)]
use winapi;
use std::u32;
use std::u64;

pub struct RenderTarget {
    device: Arc<Device>,
    pub surface: RVSurface,
    pub capabilities: RVSurfaceCapabilities,
    pub swap_chain: SwapChain,
}

impl RenderTarget {
    pub fn create_render_target_and_device(instance: Arc<Instance>, window: &Window) -> (RenderTarget, Arc<Device>) {
        let surface = RVSurface::init(&instance, window);

        let (p_device, queue_family_index) = get_usable_gpu(&instance, &surface);
        let device = Arc::new(Device::init(instance.clone(), queue_family_index, p_device));

        let surface_capabilities = surface.get_surface_capabilities(p_device, window);
        let swap_chain = SwapChain::init(&instance, &device, surface.get_present_mode(p_device), &surface, &surface_capabilities);
        (RenderTarget{
            device: device.clone(),
            surface,
            capabilities: surface_capabilities,
            swap_chain},
         device)
    }
    pub fn present(&self, rendering_complete_semaphore: &vk::Semaphore, present_index: u32) { unsafe {
        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PresentInfoKhr,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: rendering_complete_semaphore,
            swapchain_count: 1,
            p_swapchains: &self.swap_chain.handle,
            p_image_indices: &present_index,
            p_results: ptr::null_mut(),
        };
        self.swap_chain.loader.queue_present_khr(self.device.queue, &present_info).unwrap();
    }}

    pub fn next_image(&self, present_complete_semaphore: vk::Semaphore) -> u32 { unsafe {
        self.swap_chain.loader
            .acquire_next_image_khr(self.swap_chain.handle,
                                    u64::MAX,
                                    present_complete_semaphore,
                                    vk::Fence::null()).unwrap()
    }}
}

impl Drop for RenderTarget {
    fn drop(&mut self) { unsafe {
        for &image_view in self.swap_chain.image_views.iter() {
            self.device.destroy_image_view(image_view, None);
        }
        self.swap_chain.loader.destroy_swapchain_khr(self.swap_chain.handle, None);
        self.surface.loader.destroy_surface_khr(self.surface.handle, None);
    }}
}

pub struct RVSurface {
    pub loader: Surface,
    pub handle: vk::SurfaceKHR
}

pub struct RVSurfaceCapabilities {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub pre_transform: SurfaceTransformFlagsKHR,
    pub desired_image_count: uint32_t,
    pub resolution: vk::Extent2D,
    pub format: vk::SurfaceFormatKHR,
}

impl RVSurface {
    pub fn init(instance: &Arc<Instance>, window: &Window) -> RVSurface { unsafe {
        let surface = create_surface(&instance.entry, &instance.handle, window).unwrap();
        let surface_loader = Surface::new(&instance.entry, &instance.handle)
            .expect("Unable to load the surface extension");

        RVSurface {
            handle: surface,
            loader: surface_loader}
    }}

    pub fn get_surface_capabilities(&self, p_device: vk::PhysicalDevice, window: &Window) -> RVSurfaceCapabilities {
        let (width, height) = window.get_inner_size_pixels().unwrap();
        let surface_formats: Vec<vk::SurfaceFormatKHR> =
            self.loader.get_physical_device_surface_formats_khr(p_device, self.handle)
                .unwrap();
        let surface_format = surface_formats.iter()
            .map(|sfmt| {
                match sfmt.format {
                    vk::Format::Undefined => {
                        vk::SurfaceFormatKHR {
                            format: vk::Format::B8g8r8Unorm,
                            color_space: sfmt.color_space,
                        }
                    }
                    _ => sfmt.clone(),
                }
            })
            .nth(0)
            .expect("Unable to find suitable surface format.");
        let surface_capabilities: vk::SurfaceCapabilitiesKHR =
            self.loader.get_physical_device_surface_capabilities_khr(p_device, self.handle)
                .unwrap();
        let mut desired_image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0 && desired_image_count > surface_capabilities.max_image_count{
            desired_image_count = surface_capabilities.max_image_count;
        }
        let pre_transform = if surface_capabilities.supported_transforms
            .subset(vk::SURFACE_TRANSFORM_IDENTITY_BIT_KHR) {
            vk::SURFACE_TRANSFORM_IDENTITY_BIT_KHR
        } else {
            surface_capabilities.current_transform
        };
        let surface_resolution = match surface_capabilities.current_extent.width {
            u32::MAX => {
                vk::Extent2D {
                    width: width,
                    height: height,
                }
            }
            _ => surface_capabilities.current_extent.clone(),
        };

        RVSurfaceCapabilities {
            capabilities: surface_capabilities,
            pre_transform: pre_transform,
            desired_image_count: desired_image_count,
            resolution: surface_resolution,
            format: surface_format}
    }
    pub fn get_present_mode(&self, p_device: vk::PhysicalDevice) -> vk::PresentModeKHR {
        let present_modes: Vec<vk::PresentModeKHR> =
            self.loader.get_physical_device_surface_present_modes_khr(p_device, self.handle)
                .unwrap();
        present_modes.iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::Mailbox)
            .unwrap_or(vk::PresentModeKHR::Fifo)
    }
}

#[cfg(all(unix, not(target_os = "android")))]
unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(entry: &E,
                                                        instance: &I,
                                                        window: &winit::Window)
                                                        -> Result<vk::SurfaceKHR, vk::Result> {
    use winit::os::unix::WindowExt;
    let x11_display = window.get_xlib_display().unwrap();
    let x11_window = window.get_xlib_window().unwrap();
    let x11_create_info = vk::XlibSurfaceCreateInfoKHR {
        s_type: vk::StructureType::XlibSurfaceCreateInfoKhr,
        p_next: ptr::null(),
        flags: Default::default(),
        window: x11_window as vk::Window,
        dpy: x11_display as *mut vk::Display,
    };
    let xlib_surface_loader = XlibSurface::new(entry, instance)
        .expect("Unable to load xlib surface");
    xlib_surface_loader.create_xlib_surface_khr(&x11_create_info, None)
}

#[cfg(windows)]
unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(entry: &E,
                                                        instance: &I,
                                                        window: &winit::Window)
                                                        -> Result<vk::SurfaceKHR, vk::Result> {
    use winit::os::windows::WindowExt;
    let hwnd = window.get_hwnd() as *mut winapi::windef::HWND__;
    let h_instance = user32::GetWindow(hwnd, 0) as *const ();
    let win32_create_info = vk::Win32SurfaceCreateInfoKHR {
        s_type: vk::StructureType::Win32SurfaceCreateInfoKhr,
        p_next: ptr::null(),
        flags: Default::default(),
        hinstance: h_instance,
        hwnd: hwnd as *const (),
    };
    let win32_surface_loader = Win32Surface::new(entry, instance)
        .expect("Unable to load win32 surface");
    win32_surface_loader.create_win32_surface_khr(&win32_create_info, None)
}

pub struct SwapChain {
    pub loader: Swapchain,
    pub handle: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub image_count: u32,
}

impl SwapChain {
    pub fn init(instance: &Arc<Instance>,
                device: &Device,
                present_mode: vk::PresentModeKHR,
                surface: &RVSurface,
                surface_capabilities: &RVSurfaceCapabilities) -> SwapChain { unsafe {
        let loader = Swapchain::new(&instance.handle, &device.handle)
            .expect("Unable to load swapchain");
        let create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SwapchainCreateInfoKhr,
            p_next: ptr::null(),
            flags: Default::default(),
            surface: surface.handle,
            min_image_count: surface_capabilities.desired_image_count,
            image_color_space: surface_capabilities.format.color_space,
            image_format: surface_capabilities.format.format,
            image_extent: surface_capabilities.resolution.clone(),
            image_usage: vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
            image_sharing_mode: vk::SharingMode::Exclusive,
            pre_transform: surface_capabilities.pre_transform,
            composite_alpha: vk::COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
            present_mode: present_mode,
            clipped: 1,
            old_swapchain: vk::SwapchainKHR::null(),
            image_array_layers: 1,
            p_queue_family_indices: ptr::null(),
            queue_family_index_count: 0,
        };
        let swapchain = loader.create_swapchain_khr(&create_info, None)
            .unwrap();
        let images = loader.get_swapchain_images_khr(swapchain).unwrap();

        let image_views: Vec<vk::ImageView> = images.iter()
            .map(|&image| {
                let create_view_info = vk::ImageViewCreateInfo {
                    s_type: vk::StructureType::ImageViewCreateInfo,
                    p_next: ptr::null(),
                    flags: Default::default(),
                    view_type: vk::ImageViewType::Type2d,
                    format: surface_capabilities.format.format,
                    components: vk::ComponentMapping {
                        r: vk::ComponentSwizzle::R,
                        g: vk::ComponentSwizzle::G,
                        b: vk::ComponentSwizzle::B,
                        a: vk::ComponentSwizzle::A,
                    },
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::IMAGE_ASPECT_COLOR_BIT,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    image: image,
                };
                device.create_image_view(&create_view_info, None).unwrap()
            })
            .collect();
        SwapChain { loader: loader,
            handle: swapchain,
            image_count: images.len() as u32,
            images: images,
            image_views: image_views}
    } }
}
