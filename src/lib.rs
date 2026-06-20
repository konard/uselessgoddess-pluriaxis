//! *Pluriaxis* — a procedural spellcasting playground.
//!
//! The rules of magic live in the engine-agnostic [`spellvm`] crate; this crate
//! is the Bevy front end that lets you draw sigils, cast them and watch the
//! procedural effects play out. Everything is wired up through [`GamePlugin`].
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]

use crate::prelude::*;

mod core;
mod game;
mod grimoire;
pub mod prelude;
mod screens;
mod theme;

/// The single plugin that assembles the whole game.
pub struct GamePlugin;

impl Plugin for GamePlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(
      DefaultPlugins
        .set(WindowPlugin {
          primary_window: Some(Window {
            title: "Pluriaxis".into(),
            ..default()
          }),
          ..default()
        })
        .set(ImagePlugin::default_nearest()),
    );

    // A deep, near-black void so procedural neon effects pop.
    app.insert_resource(ClearColor(Color::srgb(0.015, 0.015, 0.03)));

    app.add_plugins((
      core::plugin,
      theme::plugin,
      screens::plugin,
      game::plugin,
      grimoire::plugin,
    ));
  }
}
