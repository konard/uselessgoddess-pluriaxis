//! A single 2D camera with HDR + bloom so emissive, procedural spell art glows.

use {
  crate::prelude::*,
  bevy::{
    camera::Hdr, core_pipeline::tonemapping::Tonemapping,
    post_process::bloom::Bloom,
  },
};

pub fn plugin(app: &mut App) {
  app.add_systems(Startup, spawn_camera);
}

fn spawn_camera(mut commands: Commands) {
  commands.spawn((
    Name::new("Main Camera"),
    Camera2d,
    Hdr,
    Tonemapping::TonyMcMapface,
    Bloom::NATURAL,
  ));
}
