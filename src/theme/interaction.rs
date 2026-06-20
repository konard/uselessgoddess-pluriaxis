//! Generic button feedback: tint a button by its interaction state.

use crate::{prelude::*, theme::palette};

pub fn plugin(app: &mut App) {
  app.add_systems(Update, tint_buttons);
}

/// Buttons tagged with this keep their colour (e.g. an active slot) and are not
/// repainted by the generic hover/press tinting.
#[derive(Component)]
pub struct Latched;

fn tint_buttons(
  mut q: Query<
    (&Interaction, &mut BackgroundColor),
    (Changed<Interaction>, With<Button>, Without<Latched>),
  >,
) {
  for (interaction, mut bg) in &mut q {
    bg.0 = match interaction {
      Interaction::Pressed => palette::BUTTON_PRESSED,
      Interaction::Hovered => palette::BUTTON_HOVER,
      Interaction::None => palette::BUTTON,
    };
  }
}
