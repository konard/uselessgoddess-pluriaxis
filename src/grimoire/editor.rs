//! The in-world grimoire overlay: a glyph palette to inscribe a draft spell, a
//! live preview of its sigil, and Equip/Undo/Clear actions.

use crate::{
  grimoire::{
    book::{self, Book},
    render,
  },
  prelude::*,
  theme::palette,
};

pub fn plugin(app: &mut App) {
  app.add_systems(
    OnEnter(Hud::Grimoire),
    open_book.run_if(in_state(Screen::Playing)),
  );
  app.add_systems(
    Update,
    (pick_glyph, do_action, update_draft_text, draw_draft)
      .run_if(in_state(Screen::Playing))
      .run_if(in_state(Hud::Grimoire)),
  );
}

#[derive(Component)]
struct DraftText;

#[derive(Component, Clone, Copy)]
struct PaletteGlyph(Glyph);

#[derive(Component, Clone, Copy)]
enum EditorAction {
  Undo,
  Clear,
  Equip,
}

fn open_book(mut commands: Commands, book: Res<Book>) {
  commands
    .spawn((
      Name::new("Grimoire"),
      DespawnOnExit(Hud::Grimoire),
      Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::FlexEnd,
        ..default()
      },
    ))
    .with_children(|root| {
      root
        .spawn((
          Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            padding: UiRect::all(Val::Px(18.0)),
            ..default()
          },
          BackgroundColor(palette::PANEL),
        ))
        .with_children(|panel| {
          panel.spawn((
            Text::new(
              "GRIMOIRE  ·  click glyphs to inscribe, then Equip  ·  Tab closes",
            ),
            TextFont::from_font_size(18.0),
            TextColor(palette::TEXT_DIM),
          ));
          panel.spawn((
            DraftText,
            Text::new(""),
            TextFont::from_font_size(24.0),
            TextColor(palette::TEXT),
          ));
          panel
            .spawn(Node {
              width: Val::Percent(100.0),
              flex_wrap: FlexWrap::Wrap,
              column_gap: Val::Px(6.0),
              row_gap: Val::Px(6.0),
              ..default()
            })
            .with_children(|row| {
              for g in &book.known {
                row.spawn(glyph_button(*g));
              }
            });
          panel
            .spawn(Node {
              flex_direction: FlexDirection::Row,
              column_gap: Val::Px(10.0),
              ..default()
            })
            .with_children(|row| {
              row.spawn(action_button("Undo", EditorAction::Undo));
              row.spawn(action_button("Clear", EditorAction::Clear));
              row.spawn(action_button("Equip", EditorAction::Equip));
            });
        });
    });
}

fn glyph_button(g: Glyph) -> impl Bundle {
  (
    Button,
    PaletteGlyph(g),
    Node {
      padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
      border_radius: BorderRadius::all(Val::Px(6.0)),
      ..default()
    },
    BackgroundColor(palette::BUTTON),
    children![(
      Text::new(g.name()),
      TextFont::from_font_size(16.0),
      TextColor(category_color(g)),
    )],
  )
}

fn action_button(label: &str, action: EditorAction) -> impl Bundle {
  (
    Button,
    action,
    Node {
      padding: UiRect::axes(Val::Px(18.0), Val::Px(8.0)),
      border_radius: BorderRadius::all(Val::Px(6.0)),
      ..default()
    },
    BackgroundColor(palette::BUTTON),
    children![(
      Text::new(label),
      TextFont::from_font_size(18.0),
      TextColor(palette::TEXT),
    )],
  )
}

fn category_color(g: Glyph) -> Color {
  match g.category() {
    spellvm::Category::Element => {
      let c = g
        .as_element()
        .map(|e| e.color())
        .unwrap_or(spellvm::Rgb::new(1.0, 1.0, 1.0));
      Color::srgb(c.r, c.g, c.b)
    }
    spellvm::Category::Form => palette::ACCENT,
    spellvm::Category::Modifier => Color::srgb(0.7, 0.85, 1.0),
    spellvm::Category::Operator => Color::srgb(1.0, 0.85, 0.55),
    spellvm::Category::Control => palette::TEXT_DIM,
  }
}

fn pick_glyph(
  q: Query<(&Interaction, &PaletteGlyph), Changed<Interaction>>,
  mut book: ResMut<Book>,
) {
  for (interaction, pg) in &q {
    if *interaction == Interaction::Pressed {
      book.draft.push(pg.0);
    }
  }
}

fn do_action(
  q: Query<(&Interaction, &EditorAction), Changed<Interaction>>,
  mut book: ResMut<Book>,
) {
  for (interaction, action) in &q {
    if *interaction != Interaction::Pressed {
      continue;
    }
    match action {
      EditorAction::Undo => {
        book.draft.pop();
      }
      EditorAction::Clear => book.draft.clear(),
      EditorAction::Equip => {
        if !book.draft.is_empty() {
          let spell = book.draft_spell();
          book.equipped.push(spell);
          book.selected = book.equipped.len() - 1;
          book.draft.clear();
        }
      }
    }
  }
}

fn update_draft_text(
  book: Res<Book>,
  mut q: Query<&mut Text, With<DraftText>>,
) {
  let Ok(mut text) = q.single_mut() else { return };
  let line = if book.draft.is_empty() {
    "(empty)  —  pick glyphs below".to_string()
  } else {
    let spell = book.draft_spell();
    match spell.outcome() {
      Ok(o) if !o.is_empty() => format!("{spell}  →  {}", book::summarize(&o)),
      _ => format!("{spell}  →  (incomplete)"),
    }
  };
  if text.0 != line {
    text.0 = line;
  }
}

fn draw_draft(mut gizmos: Gizmos, time: Res<Time>, book: Res<Book>) {
  let center = vec2(0.0, 120.0);
  let spin = time.elapsed_secs() * 0.12;
  if book.draft.is_empty() {
    render::draw_sigil(
      &mut gizmos,
      &Sigil::default(),
      center,
      150.0,
      spin,
      palette::ACCENT.with_alpha(0.4),
      false,
    );
    return;
  }
  let sigil = Sigil::encode(&book.draft_spell(), Style::Circle);
  render::draw_sigil(
    &mut gizmos,
    &sigil,
    center,
    170.0,
    spin,
    palette::ACCENT,
    true,
  );
}
