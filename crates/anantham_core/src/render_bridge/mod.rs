pub mod extraction;
pub mod gpu_types;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ExtractSchedule;

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RenderSchedule;

pub struct RenderBridgePlugin;

impl Plugin for RenderBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(ExtractSchedule);
        app.init_schedule(RenderSchedule);

        app.init_resource::<extraction::ExtractedVoxelData>();
        app.init_resource::<extraction::ExtractedCamera>();

        app.add_systems(
            ExtractSchedule,
            (
                extraction::extract_voxel_geometry,
                extraction::extract_camera,
            ),
        );

        app.add_systems(PostUpdate, |world: &mut World| {
            world.run_schedule(ExtractSchedule);
            world.run_schedule(RenderSchedule);
        });
    }
}
