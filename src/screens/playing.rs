//! The playground screen frame: on-field HUD text and the global hotkeys that
//! open the grimoire or leave to the menu.

use crate::{
  grimoire::book::{self, Book},
  prelude::*,
  theme::palette,
};

pub fn plugin(app: &mut App) {
  app.add_systems(OnEnter(Screen::Playing), spawn_hud);
  app.add_systems(
    Update,
    (global_input, update_readout).run_if(in_state(Screen::Playing)),
  );
}

#[derive(Component)]
struct Readout;

fn spawn_hud(mut commands: Commands) {
  commands.spawn((
    Name::new("Controls"),
    Text::new(
      "WASD move   ·   aim with mouse   ·   LMB / Space cast\n\
       1-9 select spell   ·   Tab grimoire   ·   Esc menu",
    ),
    TextFont::from_font_size(16.0),
    TextColor(palette::TEXT_DIM),
    Node {
      position_type: PositionType::Absolute,
      top: Val::Px(12.0),
      left: Val::Px(14.0),
      ..default()
    },
    DespawnOnExit(Screen::Playing),
  ));

  commands.spawn((
    Name::new("Readout"),
    Readout,
    Text::new(""),
    TextFont::from_font_size(20.0),
    TextColor(palette::TEXT),
    Node {
      position_type: PositionType::Absolute,
      bottom: Val::Px(16.0),
      left: Val::Px(14.0),
      ..default()
    },
    DespawnOnExit(Screen::Playing),
  ));
}

fn global_input(
  keys: Res<ButtonInput<KeyCode>>,
  hud: Res<State<Hud>>,
  mut next_hud: ResMut<NextState<Hud>>,
  mut next_screen: ResMut<NextState<Screen>>,
) {
  if keys.just_pressed(KeyCode::Escape) {
    if *hud.get() == Hud::Grimoire {
      next_hud.set(Hud::Field);
    } else {
      next_screen.set(Screen::Title);
    }
  }
  if keys.just_pressed(KeyCode::Tab) {
    next_hud.set(match hud.get() {
      Hud::Field => Hud::Grimoire,
      Hud::Grimoire => Hud::Field,
    });
  }
}

fn update_readout(book: Res<Book>, mut q: Query<&mut Text, With<Readout>>) {
  let Ok(mut text) = q.single_mut() else { return };
  let line = match book.equipped.get(book.selected) {
    Some(spell) => match spell.outcome() {
      Ok(o) if !o.is_empty() => {
        format!("[{}] {}  →  {}", book.selected + 1, spell, book::summarize(&o))
      }
      _ => format!("[{}] {}  →  (fizzles)", book.selected + 1, spell),
    },
    None => "no spell equipped — open the grimoire (Tab)".into(),
  };
  if text.0 != line {
    text.0 = line;
  }
}
