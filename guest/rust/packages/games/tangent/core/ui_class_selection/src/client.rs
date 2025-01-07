use std::fmt::Display;

use ambient_api::{
    core::{
        rect::components::background_color,
        text::{components::font_style, types::FontStyle},
        ui::components::focusable,
    },
    element::{use_entity_component, use_query, use_state},
    prelude::*,
    ui::{use_keyboard_input, ImageFromUrl},
};
use packages::tangent_schema::{concepts::PlayerClass, player::components as pc};
use packages::this::messages::ClassSetRequest;

pub mod packages;

#[main]
pub fn main() {
    App {}.el().spawn_interactive();
}

#[element_component]
pub fn App(hooks: &mut Hooks) -> Element {
    let player_class_id = use_entity_component(hooks, player::get_local(), pc::class_ref());

    let (toggle, set_toggle) = use_state(hooks, false);
    use_keyboard_input(hooks, {
        let set_toggle = set_toggle.clone();
        move |_, keycode, modifiers, pressed| {
            if player_class_id.is_none() {
                return;
            }

            if modifiers == ModifiersState::empty()
                && keycode == Some(VirtualKeyCode::M)
                && !pressed
            {
                set_toggle(!toggle);
            }
        }
    });

    if player_class_id.is_none() || toggle {
        ClassSelection::el(player_class_id, cb(move || set_toggle(false)))
    } else {
        Element::new()
    }
}

#[element_component]
pub fn ClassSelection(
    hooks: &mut Hooks,
    player_class_id: Option<EntityId>,
    hide: Cb<dyn Fn() + Send + Sync>,
) -> Element {
    let mut classes = use_query(hooks, PlayerClass::as_query());
    classes.sort_by_key(|(_, c)| c.name.clone());

    WindowSized::el([with_rect(
        FlowColumn::el([
            Text::el("Class Selection").header_style(),
            FlowColumn::el(classes.into_iter().map(|(id, class)| {
                Class::el(
                    id,
                    class,
                    player_class_id,
                    cb(|id| {
                        ClassSetRequest { class_id: id }.send_server_reliable();
                    }),
                    hide.clone(),
                )
            }))
            .with(space_between_items(), 4.0)
            .with(fit_horizontal(), Fit::Parent),
        ])
        .with(space_between_items(), 4.0)
        .with(fit_horizontal(), Fit::Parent),
    )
    .with_background(vec4(0.0, 0.0, 0.0, 0.5))
    .with_padding_even(STREET)])
    .with_padding_even(20.)
    .with_clickarea()
    .el()
    .with(focusable(), hooks.instance_id().to_string())
    .on_spawned(|_, _id, instance_id| {
        input::set_focus(instance_id);
    })
}

#[element_component]
pub fn Class(
    _hooks: &mut Hooks,
    class_id: EntityId,
    class: PlayerClass,
    player_class_id: Option<EntityId>,
    set_player_class_id: Cb<dyn Fn(EntityId) + Send + Sync>,
    hide: Cb<dyn Fn() + Send + Sync>,
) -> Element {
    let is_active_class = player_class_id.is_some_and(|id| id == class_id);

    let stats: &[(&str, &dyn Display)] = &[];

    with_rect(
        FlowRow::el([
            // Image
            ImageFromUrl {
                url: class.icon_url,
            }
            .el()
            .with(width(), 64.0)
            .with(height(), 64.0)
            .with(align_vertical(), Align::Center),
            // Contents
            FlowColumn::el([
                // Header
                Text::el(class.name).with(font_style(), FontStyle::Bold),
                // Description
                Text::el(class.description),
                Text::el(
                    stats
                        .iter()
                        .map(|(k, v)| format!("{k}: {v}"))
                        .collect::<Vec<_>>()
                        .join(" | "),
                ),
            ])
            .with(space_between_items(), 4.0)
            .with(align_vertical(), Align::Center),
        ])
        .with_padding_even(8.0)
        .with(space_between_items(), 8.0),
    )
    .with(fit_horizontal(), Fit::Parent)
    .with(
        background_color(),
        if is_active_class {
            vec4(0.7, 0.2, 0., 0.5)
        } else {
            vec4(0., 0., 0., 0.5)
        },
    )
    .with_clickarea()
    .on_mouse_up(move |_, _, button| {
        if button == MouseButton::Left && !is_active_class {
            set_player_class_id(class_id);
            hide();
        }
    })
    .el()
}
