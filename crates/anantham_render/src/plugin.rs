use crate::context::RenderContext;
use anantham_core::prelude::RenderSchedule;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, RawHandleWrapper, Window};

pub struct AnanthamRenderPlugin;

impl Plugin for AnanthamRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            RenderSchedule,
            (
                initialize_vulkan_system,
                draw_frame_system.after(initialize_vulkan_system),
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

fn draw_frame_system(vulkan_context: Option<ResMut<RenderContext>>) {
    if let Some(mut context) = vulkan_context {
        context.draw_frame().expect("Failed to draw frame");
    }
}
