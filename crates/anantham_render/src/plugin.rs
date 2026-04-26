use crate::context::RenderContext;
use anantham_core::prelude::RenderSchedule;
use anantham_core::render_bridge::extraction::ExtractedCamera;
use anantham_core::render_bridge::extraction::ExtractedVoxelData;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, RawHandleWrapper, Window};

pub struct AnanthamRenderPlugin;

impl Plugin for AnanthamRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            RenderSchedule,
            (
                initialize_vulkan_system,
                upload_voxel_geometry_system.after(initialize_vulkan_system),
                draw_frame_system.after(upload_voxel_geometry_system),
            ),
        );
    }
}

fn initialize_vulkan_system(
    mut commands: Commands,
    window_query: Query<(&Window, &RawHandleWrapper), With<PrimaryWindow>>,
    vulkan_context: Option<Res<RenderContext>>,
) {
    if vulkan_context.is_some() {
        return;
    }

    if let Ok((window, handle_wrappers)) = window_query.single() {
        if window.resolution.physical_width() == 0 || window.resolution.physical_height() == 0 {
            return;
        }

        info!("Initializing Vulkan Context...");
        let context = RenderContext::new(window, handle_wrappers)
            .expect("Failed to initialize Vulkan Context");

        commands.insert_resource(context);
    }
}

fn draw_frame_system(vulkan_context: Option<ResMut<RenderContext>>, camera: Res<ExtractedCamera>) {
    if let Some(mut context) = vulkan_context {
        context.draw_frame(&camera).expect("Failed to draw frame");
    }
}

pub fn upload_voxel_geometry_system(
    render_context: Option<ResMut<RenderContext>>,
    mut extracted_data: ResMut<ExtractedVoxelData>,
) {
    let Some(mut ctx) = render_context else {
        return;
    };

    if !extracted_data.requires_upload {
        return;
    }

    let mut all_meshlets = Vec::new();
    let mut all_quads = Vec::new();
    let mut current_quad_offset = 0;

    for (meshlets, quads) in extracted_data.active_chunks.values() {
        for meshlet in meshlets {
            let mut adjusted_meshlet = *meshlet;
            adjusted_meshlet.quad_offset += current_quad_offset;
            all_meshlets.push(adjusted_meshlet);
        }

        current_quad_offset += quads.len() as u32;
        all_quads.extend(quads.iter().cloned());
    }

    if !all_meshlets.is_empty() {
        ctx.scene.meshlet_count = all_meshlets.len() as u32;
        ctx.scene.meshlet_buffer.write_mapped(&all_meshlets);
        ctx.scene.quad_buffer.write_mapped(&all_quads);

        info!(
            "Uploaded {} meshlets and {} quads to GPU.",
            all_meshlets.len(),
            all_quads.len()
        );
    }

    extracted_data.requires_upload = false;
}
