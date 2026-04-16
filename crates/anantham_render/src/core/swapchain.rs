use super::device::VulkanDevice;
use ash::{khr, vk};
use bevy::window::Window;
use std::error::Error;

pub struct SwapchainSetup {
    pub ext: khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
}

impl SwapchainSetup {
    pub fn new(window: &Window, vkd: &VulkanDevice) -> Result<Self, Box<dyn Error>> {
        let surface_ext = &vkd.surface_ext;
        let surface = vkd.surface;
        let pdevice = vkd.physical_device;

        unsafe {
            let formats = surface_ext.get_physical_device_surface_formats(pdevice, surface)?;
            let format = formats
                .iter()
                .find(|f| {
                    f.format == vk::Format::B8G8R8A8_UNORM
                        && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                })
                .unwrap_or(&formats[0])
                .format;

            let capabilities =
                surface_ext.get_physical_device_surface_capabilities(pdevice, surface)?;

            if capabilities.current_extent.width == 0 || capabilities.current_extent.height == 0 {
                return Err(Box::from(
                    "Wayland surface unmapped (0x0). Waiting for Compositor...",
                ));
            }

            let extent = if capabilities.current_extent.width != u32::MAX {
                capabilities.current_extent
            } else {
                let phys_w = window.resolution.physical_width();
                let phys_h = window.resolution.physical_height();
                vk::Extent2D {
                    width: phys_w.clamp(
                        capabilities.min_image_extent.width,
                        capabilities.max_image_extent.width,
                    ),
                    height: phys_h.clamp(
                        capabilities.min_image_extent.height,
                        capabilities.max_image_extent.height,
                    ),
                }
            };

            let present_modes =
                surface_ext.get_physical_device_surface_present_modes(pdevice, surface)?;
            let present_mode = present_modes
                .into_iter()
                .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
                .unwrap_or(vk::PresentModeKHR::FIFO);

            let mut image_count = capabilities.min_image_count + 1;
            if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
                image_count = capabilities.max_image_count;
            }

            let create_info = vk::SwapchainCreateInfoKHR::default()
                .surface(surface)
                .min_image_count(image_count)
                .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
                .image_format(format)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true);

            let ext = khr::swapchain::Device::new(&vkd.instance, &vkd.device);
            let swapchain = ext.create_swapchain(&create_info, None)?;

            let images = ext.get_swapchain_images(swapchain)?;
            let mut image_views = Vec::with_capacity(images.len());

            for &image in &images {
                let view_info = vk::ImageViewCreateInfo::default()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::R,
                        g: vk::ComponentSwizzle::G,
                        b: vk::ComponentSwizzle::B,
                        a: vk::ComponentSwizzle::A,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .image(image);

                image_views.push(vkd.device.create_image_view(&view_info, None)?);
            }

            Ok(Self {
                ext,
                swapchain,
                format,
                extent,
                images,
                image_views,
            })
        }
    }
}
