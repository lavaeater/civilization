use bevy::ecs::system::IntoObserverSystem;
use bevy::ecs::spawn::Spawn;
use bevy::prelude::*;

use bevy::feathers::controls::{button, ButtonProps, ButtonVariant};
use bevy::feathers::rounded_corners::RoundedCorners;
use bevy::feathers::theme::ThemedText;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::Activate;

use crate::{ButtonBuilder, UIBuilder};

impl<'a, 'w, 's> ButtonBuilder<'a, 'w, 's> {
    pub fn text(&mut self, text: impl Into<String>) -> &mut Self {
        if let Some(text_entity) = self.text_entity {
            self.ui.commands.entity(text_entity).insert(Text::new(text.into()));
        }
        self
    }

    pub fn font(&mut self, font: Handle<Font>) -> &mut Self {
        if let Some(text_entity) = self.text_entity {
            self.ui.commands.entity(text_entity).entry::<TextFont>()
                .and_modify(move |mut tf| { tf.font = font.clone(); });
        }
        self
    }

    pub fn font_size(&mut self, size: f32) -> &mut Self {
        if let Some(text_entity) = self.text_entity {
            self.ui.commands.entity(text_entity).entry::<TextFont>()
                .and_modify(move |mut tf| { tf.font_size = size; });
        }
        self
    }

    pub fn text_color(&mut self, color: Color) -> &mut Self {
        if let Some(text_entity) = self.text_entity {
            self.ui.commands.entity(text_entity).insert(TextColor(color));
        }
        self
    }

    pub fn width(&mut self, width: Val) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.width = width; });
        self
    }

    pub fn width_px(&mut self, width: f32) -> &mut Self { self.width(Val::Px(width)) }
    pub fn width_percent(&mut self, width: f32) -> &mut Self { self.width(Val::Percent(width)) }

    pub fn height(&mut self, height: Val) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.height = height; });
        self
    }

    pub fn height_px(&mut self, height: f32) -> &mut Self { self.height(Val::Px(height)) }
    pub fn height_percent(&mut self, height: f32) -> &mut Self { self.height(Val::Percent(height)) }

    pub fn size_px(&mut self, width: f32, height: f32) -> &mut Self {
        self.width_px(width).height_px(height)
    }

    pub fn bg_color(&mut self, color: Color) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).insert(BackgroundColor(color));
        self
    }

    pub fn border(&mut self, border: UiRect) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.border = border; });
        self
    }

    pub fn border_all_px(&mut self, width: f32) -> &mut Self { self.border(UiRect::all(Val::Px(width))) }

    pub fn border_color(&mut self, color: Color) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).insert(BorderColor::all(color));
        self
    }

    pub fn border_radius(&mut self, radius: BorderRadius) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.border_radius = radius; });
        self
    }

    pub fn border_radius_all_px(&mut self, radius: f32) -> &mut Self {
        self.border_radius(BorderRadius::all(Val::Px(radius)))
    }

    pub fn border_radius_all_percent(&mut self, radius: f32) -> &mut Self {
        self.border_radius(BorderRadius::all(Val::Percent(radius)))
    }

    pub fn justify_content(&mut self, justify: JustifyContent) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.justify_content = justify; });
        self
    }

    pub fn align_items(&mut self, align: AlignItems) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.align_items = align; });
        self
    }

    pub fn padding(&mut self, padding: UiRect) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.padding = padding; });
        self
    }

    pub fn padding_all_px(&mut self, value: f32) -> &mut Self { self.padding(UiRect::all(Val::Px(value))) }

    pub fn margin(&mut self, margin: UiRect) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.margin = margin; });
        self
    }

    pub fn margin_all_px(&mut self, value: f32) -> &mut Self { self.margin(UiRect::all(Val::Px(value))) }

    pub fn insert<T: Component>(&mut self, component: T) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).insert(component);
        self
    }

    pub fn disabled(&mut self) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).insert(InteractionDisabled);
        self
    }

    pub fn enabled(&mut self) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).remove::<InteractionDisabled>();
        self
    }
}

// ============================================================================
// Feathers-style button helpers using observe() pattern
// ============================================================================

impl<'w, 's> UIBuilder<'w, 's> {
    /// Add a Feathers-style button with an observer for the Activate event.
    /// 
    /// # Example
    /// ```ignore
    /// ui.feathers_button("Click Me", |_: On<Activate>, mut commands: Commands| {
    ///     info!("Button clicked!");
    /// });
    /// ```
    pub fn feathers_button<M>(
        &mut self,
        text: impl Into<String>,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self {
        self.feathers_button_with_props(text, ButtonProps::default(), (), handler)
    }

    /// Add a Feathers-style button with custom props and an observer.
    /// 
    /// # Example
    /// ```ignore
    /// ui.feathers_button_with_props(
    ///     "Primary",
    ///     ButtonProps { variant: ButtonVariant::Primary, ..default() },
    ///     (),
    ///     |_: On<Activate>| { info!("Clicked!"); }
    /// );
    /// ```
    pub fn feathers_button_with_props<B, M>(
        &mut self,
        text: impl Into<String>,
        props: ButtonProps,
        extra_components: B,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self
    where
        B: Bundle,
    {
        let text_str = text.into();
        let original_entity = self.current_entity;
        let original_stack_len = self.parent_stack.len();

        self.child();
        let button_entity = self.current_entity;

        // Spawn the Feathers button bundle with observer
        let button_bundle = button(
            props,
            extra_components,
            Spawn((Text::new(text_str), ThemedText)),
        );

        self.commands
            .entity(button_entity)
            .insert(button_bundle)
            .observe(handler);

        // Restore state
        self.current_entity = original_entity;
        while self.parent_stack.len() > original_stack_len {
            self.parent_stack.pop_back();
        }

        self
    }

    /// Add a Feathers-style primary button with an observer.
    pub fn feathers_button_primary<M>(
        &mut self,
        text: impl Into<String>,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self {
        self.feathers_button_with_props(
            text,
            ButtonProps {
                variant: ButtonVariant::Primary,
                ..default()
            },
            (),
            handler,
        )
    }

    /// Add a disabled Feathers-style button with an observer.
    pub fn feathers_button_disabled<M>(
        &mut self,
        text: impl Into<String>,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self {
        self.feathers_button_with_props(text, ButtonProps::default(), InteractionDisabled, handler)
    }

    /// Add a Feathers-style button with a marker component and an observer.
    /// 
    /// # Example
    /// ```ignore
    /// ui.feathers_button_marked("Save", SaveButton, |_: On<Activate>| {
    ///     info!("Save clicked!");
    /// });
    /// ```
    pub fn feathers_button_marked<T, M>(
        &mut self,
        text: impl Into<String>,
        marker: T,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self
    where
        T: Component,
    {
        self.feathers_button_with_props(text, ButtonProps::default(), marker, handler)
    }

    /// Add a Feathers-style button with custom rounded corners.
    pub fn feathers_button_corners<M>(
        &mut self,
        text: impl Into<String>,
        corners: RoundedCorners,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self {
        self.feathers_button_with_props(
            text,
            ButtonProps {
                corners,
                ..default()
            },
            (),
            handler,
        )
    }

    pub fn with_rounded_corners(&mut self, radius: f32) -> &mut Self {
        self.commands.entity(self.current_entity).insert(RoundedCorners::all(radius));
        self
    }
}
