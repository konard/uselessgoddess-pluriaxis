//! The main menu, with a slowly turning sigil behind it.

use crate::{
  grimoire::render,
  prelude::*,
  theme::{palette, widget},
};

pub fn plugin(app: &mut App) {
  app.add_systems(OnEnter(Screen::Title), spawn_menu);
  app.add_systems(
    Update,
    (press_buttons, draw_backdrop).run_if(in_state(Screen::Title)),
  );
}

#[derive(Component, Clone, Copy)]
enum TitleAction {
  Play,
  Quit,
}

fn spawn_menu(mut commands: Commands) {
  commands
    .spawn((widget::screen_root("Title"), DespawnOnExit(Screen::Title)))
    .with_children(|root| {
      root.spawn(widget::panel()).with_children(|p| {
        p.spawn(widget::heading("Pluriaxis"));
        p.spawn(widget::hint(
          "Draw spells as sigils. Cast light into the dark.",
        ));
        p.spawn(widget::button("Enter the Circle", TitleAction::Play));
        p.spawn(widget::button("Quit", TitleAction::Quit));
      });
    });
}

fn press_buttons(
  q: Query<(&Interaction, &TitleAction), Changed<Interaction>>,
  mut next: ResMut<NextState<Screen>>,
  mut exit: MessageWriter<AppExit>,
) {
  for (interaction, action) in &q {
    if *interaction != Interaction::Pressed {
      continue;
    }
    match action {
      TitleAction::Play => next.set(Screen::Playing),
      TitleAction::Quit => {
        exit.write(AppExit::Success);
      }
    }
  }
}

fn draw_backdrop(mut gizmos: Gizmos, time: Res<Time>) {
  // A sample spell, inscribed and slowly turning behind the menu.
  let spell = Spell::new([
    Glyph::Fire,
    Glyph::Fuse,
    Glyph::Air,
    Glyph::Bolt,
    Glyph::Amplify,
    Glyph::Split,
  ]);
  let sigil = Sigil::encode(&spell, Style::Circle);
  let t = time.elapsed_secs();
  render::draw_sigil(
    &mut gizmos,
    &sigil,
    Vec2::ZERO,
    240.0,
    t * 0.12,
    palette::ACCENT,
    true,
  );
  render::draw_sigil(
    &mut gizmos,
    &sigil,
    Vec2::ZERO,
    150.0,
    -t * 0.2,
    palette::glow(spellvm::Rgb::new(0.4, 0.7, 1.0), 1.4),
    false,
  );
}
