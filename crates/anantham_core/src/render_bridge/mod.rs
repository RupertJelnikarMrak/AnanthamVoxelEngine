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

        app.add_systems(PostUpdate, |world: &mut World| {
            world.run_schedule(RenderSchedule);
        });
    }
}
