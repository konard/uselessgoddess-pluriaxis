//! Small builders for the handful of UI shapes the game needs, so screens read
//! declaratively.

use crate::{prelude::*, theme::palette};

/// A full-screen, centred column.
pub fn screen_root(name: &'static str) -> impl Bundle {
  (
    Name::new(name),
    Node {
      width: Val::Percent(100.0),
      height: Val::Percent(100.0),
      flex_direction: FlexDirection::Column,
      align_items: AlignItems::Center,
      justify_content: JustifyContent::Center,
      row_gap: Val::Px(16.0),
      ..default()
    },
  )
}

/// A bordered, padded panel that lays its children out in a column.
pub fn panel() -> impl Bundle {
  (
    Node {
      flex_direction: FlexDirection::Column,
      align_items: AlignItems::Center,
      row_gap: Val::Px(12.0),
      padding: UiRect::all(Val::Px(24.0)),
      border_radius: BorderRadius::all(Val::Px(12.0)),
      ..default()
    },
    BackgroundColor(palette::PANEL),
  )
}

/// Large accent heading text.
pub fn heading(text: impl Into<String>) -> impl Bundle {
  (Text::new(text), TextFont::from_font_size(56.0), TextColor(palette::ACCENT))
}

/// Dim, smaller helper text.
pub fn hint(text: impl Into<String>) -> impl Bundle {
  (
    Text::new(text),
    TextFont::from_font_size(18.0),
    TextColor(palette::TEXT_DIM),
  )
}

/// A clickable button carrying an `action` marker component, with a centred
/// label. Query `(&Interaction, &Action)` to react to presses.
pub fn button<A: Component>(text: impl Into<String>, action: A) -> impl Bundle {
  (
    Button,
    action,
    Node {
      min_width: Val::Px(220.0),
      padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
      justify_content: JustifyContent::Center,
      align_items: AlignItems::Center,
      border_radius: BorderRadius::all(Val::Px(8.0)),
      ..default()
    },
    BackgroundColor(palette::BUTTON),
    children![(
      Text::new(text),
      TextFont::from_font_size(24.0),
      TextColor(palette::TEXT),
    )],
  )
}
