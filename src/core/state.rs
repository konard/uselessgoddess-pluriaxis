//! App states and the shared system ordering every feature hangs off of.

use crate::prelude::*;

pub fn plugin(app: &mut App) {
  app.init_state::<Screen>();
  app.init_state::<Hud>();

  // A predictable per-frame order: advance timers, read input, then react.
  app.configure_sets(
    Update,
    (AppSystems::TickTimers, AppSystems::RecordInput, AppSystems::Update)
      .chain(),
  );

  // The book is always closed when we step onto (or leave) the field.
  app.add_systems(OnEnter(Screen::Playing), close_hud);
  app.add_systems(OnExit(Screen::Playing), close_hud);
}

/// The top-level screen the player is looking at.
#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Screen {
  /// The main menu.
  #[default]
  Title,
  /// The spellcasting playground.
  Playing,
}

/// What is overlaid on the playground: nothing, or the grimoire editor.
#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Hud {
  /// Plain field — the player runs around and casts.
  #[default]
  Field,
  /// The grimoire is open; gameplay is suspended while you compose spells.
  Grimoire,
}

/// Coarse system ordering buckets, configured to run in this order each frame.
#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AppSystems {
  /// Tick timers and cooldowns.
  TickTimers,
  /// Sample input into components/resources.
  RecordInput,
  /// React to input and advance the simulation.
  Update,
}

fn close_hud(mut next: ResMut<NextState<Hud>>) {
  next.set(Hud::Field);
}
