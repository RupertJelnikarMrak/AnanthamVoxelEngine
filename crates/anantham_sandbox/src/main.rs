use anantham_core::plugin::AnanthamCorePlugin;
use anantham_core::prelude::*;

fn main() -> AppExit {
    let mut app = App::new();

    app.add_plugins(AnanthamCorePlugin);

    app.run()
}
