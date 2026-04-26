use crate::core::pipeline::VoxelPipeline;
use crate::core::{device::VulkanDevice, swapchain::SwapchainSetup, sync::SyncSetup};
use crate::resource::allocator::GpuAllocator;
use crate::resource::image::DepthBuffer;
use crate::resource::scene::GpuVoxelScene;
use anantham_core::render_bridge::extraction::ExtractedCamera;

use ash::vk;
use bevy::prelude::*;
use bevy::window::{RawHandleWrapper, Window};
use std::error::Error;
use std::mem::ManuallyDrop;

#[derive(Resource)]
pub struct RenderContext {
    pub vkd: VulkanDevice,
    pub swapchain: SwapchainSetup,
    pub sync: SyncSetup,
    pub allocator: GpuAllocator,
    pub scene: ManuallyDrop<GpuVoxelScene>,
    pub pipeline: VoxelPipeline,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set: vk::DescriptorSet,
    pub depth_buffer: ManuallyDrop<DepthBuffer>,
}

impl RenderContext {
    pub fn new(window: &Window, handle_wrapper: &RawHandleWrapper) -> Result<Self, Box<dyn Error>> {
        debug!("Starting Vulkan boot sequence...");

        let vkd = VulkanDevice::new(handle_wrapper)?;
        let swapchain = SwapchainSetup::new(window, &vkd)?;
        let sync = SyncSetup::new(&vkd)?;
        let mut allocator = GpuAllocator::new(
            vkd.instance.clone(),
            vkd.device.clone(),
            vkd.physical_device,
        )?;
        let scene = GpuVoxelScene::new(&vkd.device, &mut allocator)?;
        let pipeline = VoxelPipeline::new(&vkd.device, swapchain.format)?;

        let pool_sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(2)];

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(2);

        let descriptor_pool = unsafe { vkd.device.create_descriptor_pool(&pool_info, None)? };

        let layouts = [pipeline.descriptor_set_layout];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_set = unsafe { vkd.device.allocate_descriptor_sets(&alloc_info)?[0] };

        let quad_info = vk::DescriptorBufferInfo::default()
            .buffer(scene.quad_buffer.buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);

        let meshlet_info = vk::DescriptorBufferInfo::default()
            .buffer(scene.meshlet_buffer.buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);

        let write_quads = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0) // Matches layout(binding = 0) in GLSL
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&quad_info));

        let write_meshlets = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(1) // Matches layout(binding = 1) in GLSL
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&meshlet_info));

        unsafe {
            vkd.device
                .update_descriptor_sets(&[write_quads, write_meshlets], &[]);
        }

        let depth_buffer = DepthBuffer::new(&vkd.device, &mut allocator, swapchain.extent)?;

        info!("Base Vulkan Context fully initialized");

        Ok(Self {
            vkd,
            swapchain,
            sync,
            allocator,
            scene: ManuallyDrop::new(scene),
            pipeline,
            descriptor_pool,
            descriptor_set,
            depth_buffer: ManuallyDrop::new(depth_buffer),
        })
    }

    pub fn draw_frame(&mut self, camera: &ExtractedCamera) -> Result<(), Box<dyn Error>> {
        self.record_and_submit_commands(camera)?;
        Ok(())
    }

    fn record_and_submit_commands(
        &mut self,
        camera: &ExtractedCamera,
    ) -> Result<(), Box<dyn Error>> {
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

            let depth_barrier = vk::ImageMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(
                    vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
                        | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                )
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .image(self.depth_buffer.image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            device.cmd_pipeline_barrier(
                self.sync.command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_memory_barrier, depth_barrier],
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

            let depth_attachment_info = vk::RenderingAttachmentInfo::default()
                .image_view(self.depth_buffer.view)
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .clear_value(vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 0.0,
                        stencil: 0,
                    },
                });

            let rendering_info = vk::RenderingInfo::default()
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.swapchain.extent,
                })
                .layer_count(1)
                .color_attachments(std::slice::from_ref(&color_attachment_info))
                .depth_attachment(&depth_attachment_info);

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

            device.cmd_bind_pipeline(
                self.sync.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );

            device.cmd_bind_descriptor_sets(
                self.sync.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline_layout,
                0,
                std::slice::from_ref(&self.descriptor_set),
                &[],
            );

            device.cmd_push_constants(
                self.sync.command_buffer,
                self.pipeline.pipeline_layout,
                vk::ShaderStageFlags::MESH_EXT,
                0,
                bytemuck::bytes_of(&camera.view_proj),
            );

            if self.scene.meshlet_count > 0 {
                self.vkd.mesh_ext.cmd_draw_mesh_tasks(
                    self.sync.command_buffer,
                    self.scene.meshlet_count,
                    1,
                    1,
                );
            }

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

            let depth_buffer = ManuallyDrop::take(&mut self.depth_buffer);
            depth_buffer.destroy(device, &mut self.allocator);

            self.pipeline.destroy(device);
            device.destroy_descriptor_pool(self.descriptor_pool, None);

            let scene = ManuallyDrop::take(&mut self.scene);
            scene.destroy(device, &mut self.allocator);

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
