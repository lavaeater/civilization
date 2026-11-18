//! Bevy 0.20 made the legacy `bevy_ui::widget::Button` private (only a deprecated
//! alias is left) and points to `ui_widgets::Button` instead. That widget does not
//! require `Interaction`, which the hand-rolled button systems in the calamity, city
//! and trade UIs still poll via `Changed<Interaction>`. This bundle is the old
//! `Button`'s required components spelled out, on top of the new widget, so those
//! systems keep working unchanged.
//!
//! `Interaction` itself is deprecated and slated for removal after 0.20; the
//! long-term fix is to move those systems to `On<Activate>` observers.

use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::ui_widgets::Button;

#[allow(deprecated)]
pub fn interaction_button() -> impl Bundle {
    (Button, Interaction::default(), FocusPolicy::Block)
}
