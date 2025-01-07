use std::str::FromStr;

use ambient_core::window::cursor_position;
use ambient_ecs::{generated::messages, query_mut, SystemGroup, World};
use ambient_element::{element_component, use_runtime_message, use_state, Element, Hooks};
use ambient_input::{player_prev_raw_input, player_raw_input, PlayerRawInput};
use ambient_network::client::client_state;
use ambient_shared_types::VirtualKeyCode;
use glam::Vec2;

pub fn systems_final() -> SystemGroup {
    SystemGroup::new(
        "player/client_systems_final",
        vec![
            query_mut((player_prev_raw_input(), player_raw_input()), ()).to_system(
                |q, world, qs, _| {
                    for (_, (prev, input), ()) in q.iter(world, qs) {
                        input.mouse_delta = glam::Vec2::ZERO;
                        *prev = input.clone();
                    }
                },
            ),
        ],
    )
}

#[element_component]
pub fn PlayerRawInputHandler(hooks: &mut Hooks) -> Element {
    const PIXELS_PER_LINE: f32 = 5.0;

    // Assume window has focus
    let (has_focus, set_has_focus) = use_state(hooks, true);
    use_runtime_message::<messages::WindowFocusChange>(hooks, move |world, event| {
        set_has_focus(event.focused);

        if !event.focused {
            process_input(world, true, |input, _| input.clear());
        }
    });

    use_runtime_message::<messages::WindowKeyboardInput>(hooks, move |world, event| {
        process_input(world, has_focus, |input, _| {
            if let Some(keycode) = event.keycode.as_deref() {
                let keycode = VirtualKeyCode::from_str(keycode).unwrap();
                if event.pressed {
                    input.keys.insert(keycode);
                } else {
                    input.keys.remove(&keycode);
                }
            }
        });
    });

    use_runtime_message::<messages::WindowMouseInput>(hooks, move |world, event| {
        process_input(world, has_focus, |input, _| {
            if event.pressed {
                input.mouse_buttons.insert(event.button.into());
            } else {
                input.mouse_buttons.remove(&event.button.into());
            }
        });
    });

    use_runtime_message::<messages::WindowMouseMotion>(hooks, move |world, msg| {
        process_input(world, has_focus, |input, mouse_pos| {
            input.mouse_position = mouse_pos;
            input.mouse_delta += msg.delta;
        });
    });

    use_runtime_message::<messages::WindowMouseWheel>(hooks, move |world, event| {
        process_input(world, has_focus, |input, _| {
            let delta = event.delta;
            input.mouse_wheel += match event.pixels {
                false => delta.y * PIXELS_PER_LINE,
                true => delta.y,
            };
        });
    });

    fn process_input(
        ui_world: &World,
        has_focus: bool,
        processor: impl Fn(&mut PlayerRawInput, Vec2),
    ) {
        if !has_focus {
            return;
        }

        let Some(Some(gc)) = ui_world.resource_opt(client_state()).cloned() else {
            return;
        };
        gc.with_physics_world(|w| {
            let mouse_pos = *w.resource(cursor_position());
            let input = w.resource_mut(player_raw_input());
            processor(input, mouse_pos);
        });
    }

    Element::new()
}
