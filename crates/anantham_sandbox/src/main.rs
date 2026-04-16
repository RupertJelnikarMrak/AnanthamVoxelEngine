use anantham_core::plugin::AnanthamCorePlugin;
use anantham_render::plugin::AnanthamRenderPlugin;
use bevy::prelude::*;

fn main() -> AppExit {
    let mut app = App::new();

    app.add_plugins((AnanthamCorePlugin, AnanthamRenderPlugin));

    app.run()
}
