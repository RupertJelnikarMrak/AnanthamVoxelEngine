use crate::resource::allocator::GpuAllocator;
use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc};
use std::error::Error;

pub struct DepthBuffer {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub allocation: Allocation,
    pub format: vk::Format,
}

impl DepthBuffer {
    pub fn new(
        device: &ash::Device,
        allocator: &mut GpuAllocator,
        extent: vk::Extent2D,
    ) -> Result<Self, Box<dyn Error>> {
        let format = vk::Format::D32_SFLOAT;

        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let image = unsafe { device.create_image(&image_info, None)? };
        let requirements = unsafe { device.get_image_memory_requirements(image) };

        let allocation = allocator.allocate(&AllocationCreateDesc {
            name: "Depth Buffer",
            requirements,
            location: gpu_allocator::MemoryLocation::GpuOnly,
            linear: false,
            allocation_scheme: gpu_allocator::vulkan::AllocationScheme::GpuAllocatorManaged,
        })?;

        unsafe {
            device.bind_image_memory(image, allocation.memory(), allocation.offset())?;
        }

        let view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let view = unsafe { device.create_image_view(&view_info, None)? };

        Ok(Self {
            image,
            view,
            allocation,
            format,
        })
    }

    pub fn destroy(self, device: &ash::Device, allocator: &mut GpuAllocator) {
        unsafe {
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.image, None);
        }
        allocator.free(self.allocation).unwrap();
    }
}
