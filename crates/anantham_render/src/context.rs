use crate::core::{device::VulkanDevice, swapchain::SwapchainSetup, sync::SyncSetup};
use crate::resource::allocator::GpuAllocator;

use ash::vk;
use bevy::prelude::*;
use bevy::window::{RawHandleWrapper, Window};
use std::error::Error;

#[derive(Resource)]
pub struct RenderContext {
    pub vkd: VulkanDevice,
    pub swapchain: SwapchainSetup,
    pub sync: SyncSetup,
    pub allocator: GpuAllocator,
}

impl RenderContext {
    pub fn new(window: &Window, handle_wrapper: &RawHandleWrapper) -> Result<Self, Box<dyn Error>> {
        debug!("Starting Vulkan boot sequence...");

        let vkd = VulkanDevice::new(handle_wrapper)?;
        let swapchain = SwapchainSetup::new(window, &vkd)?;
        let sync = SyncSetup::new(&vkd)?;

        let allocator = GpuAllocator::new(
            vkd.instance.clone(),
            vkd.device.clone(),
            vkd.physical_device,
        )?;

        info!("Base Vulkan Context fully initialized");

        Ok(Self {
            vkd,
            swapchain,
            sync,
            allocator,
        })
    }

    pub fn draw_frame(&mut self) -> Result<(), Box<dyn Error>> {
        self.record_and_submit_commands()?;
        Ok(())
    }

    fn record_and_submit_commands(&mut self) -> Result<(), Box<dyn Error>> {
        let device = &self.vkd.device;
        let swapchain_ext = &self.swapchain.ext;

        unsafe {
            device.wait_for_fences(&[self.sync.in_flight], true, u64::MAX)?;
            device.reset_fences(&[self.sync.in_flight])?;

            let (image_index, _) = swapchain_ext.acquire_next_image(
                self.swapchain.swapchain,
                u64::MAX,
                self.sync.image_available,
                vk::Fence::null(),
            )?;

            device.reset_command_buffer(
                self.sync.command_buffer,
                vk::CommandBufferResetFlags::empty(),
            )?;

            let begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            device.begin_command_buffer(self.sync.command_buffer, &begin_info)?;

            let image = self.swapchain.images[image_index as usize];
            let image_view = self.swapchain.image_views[image_index as usize];

            let mut image_memory_barrier = vk::ImageMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .image(image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            device.cmd_pipeline_barrier(
                self.sync.command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_memory_barrier],
            );

            let clear_value = vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1, 0.1, 0.15, 1.0], // Default dark slate clear color
                },
            };
            let color_attachment_info = vk::RenderingAttachmentInfo::default()
                .image_view(image_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(clear_value);

            let rendering_info = vk::RenderingInfo::default()
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.swapchain.extent,
                })
                .layer_count(1)
                .color_attachments(std::slice::from_ref(&color_attachment_info));

            device.cmd_begin_rendering(self.sync.command_buffer, &rendering_info);

            // Dynamic rendering requires viewport and scissor to be set even if empty
            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: self.swapchain.extent.width as f32,
                height: self.swapchain.extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };
            device.cmd_set_viewport(self.sync.command_buffer, 0, &[viewport]);

            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            };
            device.cmd_set_scissor(self.sync.command_buffer, 0, &[scissor]);

            // --- Extracted Render Nodes & Pipelines will be executed here ---

            device.cmd_end_rendering(self.sync.command_buffer);

            // Transition image layout for presentation
            image_memory_barrier.src_access_mask = vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
            image_memory_barrier.dst_access_mask = vk::AccessFlags::empty();
            image_memory_barrier.old_layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
            image_memory_barrier.new_layout = vk::ImageLayout::PRESENT_SRC_KHR;

            device.cmd_pipeline_barrier(
                self.sync.command_buffer,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&image_memory_barrier),
            );

            device.end_command_buffer(self.sync.command_buffer)?;

            let wait_semaphores = [self.sync.image_available];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [self.sync.command_buffer];
            let signal_semaphores = [self.sync.render_finished];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);

            device.queue_submit(self.vkd.graphics_queue, &[submit_info], self.sync.in_flight)?;

            let swapchains = [self.swapchain.swapchain];
            let image_indices = [image_index];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            swapchain_ext.queue_present(self.vkd.graphics_queue, &present_info)?;
        }
        Ok(())
    }
}

impl Drop for RenderContext {
    fn drop(&mut self) {
        unsafe {
            let device = &self.vkd.device;
            let _ = device.device_wait_idle();

            device.destroy_fence(self.sync.in_flight, None);
            device.destroy_semaphore(self.sync.render_finished, None);
            device.destroy_semaphore(self.sync.image_available, None);
            device.destroy_command_pool(self.sync.command_pool, None);

            for &view in &self.swapchain.image_views {
                device.destroy_image_view(view, None);
            }
            self.swapchain
                .ext
                .destroy_swapchain(self.swapchain.swapchain, None);

            info!("Base Vulkan Context destroyed cleanly.");
        }
    }
}
