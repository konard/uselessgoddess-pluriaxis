// Configuring Bevy lints from code requires this attribute.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Hide the console window on Windows in non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use {bevy::prelude::*, pluriaxis::GamePlugin};

fn main() -> AppExit {
  App::new().add_plugins(GamePlugin).run()
}
