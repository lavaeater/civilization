//! Clicks on the hand-rolled game buttons (calamity, city, trade, civ-card and sandbox
//! UIs).
//!
//! Bevy 0.20 deprecated `Interaction` in favour of `ui_widgets::Button`, which reports
//! a click as an `Activate` entity event, and picking's `Hovered`/`Pressed` for state.
//! The button systems here are ordinary state-gated systems that look up an action
//! component on the clicked entity, so one global observer forwards every `Activate`
//! as a [`ButtonActivated`] message they can read. Hover and press colours come from
//! lava_ui_builder's `InteractionPalette`, which reads `Hovered`/`Pressed`.
//!
//! `Activate` fires when the pointer is released over the button (or on ENTER/SPACE
//! while focused), where `Interaction::Pressed` used to fire on press.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

/// A clickable game button: the headless widget plus hover tracking for its palette.
pub fn game_button() -> impl Bundle {
    (Button, Hovered::default())
}

/// A button entity was clicked (released over it, or activated from the keyboard).
#[derive(Message, Clone, Copy, Debug)]
pub struct ButtonActivated(pub Entity);

pub struct ButtonActivationPlugin;

impl Plugin for ButtonActivationPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ButtonActivated>().add_observer(
            |activate: On<Activate>, mut activated: MessageWriter<ButtonActivated>| {
                activated.write(ButtonActivated(activate.entity));
            },
        );
    }
}
