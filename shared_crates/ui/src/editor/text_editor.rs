#[cfg(feature = "guest")]
use std::time::Instant;

use ambient_cb::{cb, Cb};
use ambient_element::{
    element_component, to_owned, use_frame, use_ref_with, use_rerender_signal, use_runtime_message,
    use_spawn, use_state, Element, ElementComponentExt, Hooks,
};
use ambient_guest_bridge::{
    core::{
        layout::components::{height, min_height, min_width, width},
        messages,
        rect::components::background_color,
        rendering::components::color,
        text::components::text,
        transform::components::translation,
        ui::components::focusable,
    },
    window::set_cursor,
};
use ambient_shared_types::{CursorIcon, VirtualKeyCode};
#[cfg(feature = "native")]
use ambient_sys::time::Instant;
use glam::*;
use itertools::Itertools;

use super::{Editor, EditorOpts};
use crate::{
    layout::FlowRow, text::Text, use_focus, use_keyboard_input, with_rect, Rectangle, UIBase, UIExt,
};

/// A text editor.
#[element_component]
pub fn TextEditor(
    hooks: &mut Hooks,
    /// The string to edit.
    value: String,
    /// Callback for when the value is changed.
    on_change: Cb<dyn Fn(String) + Sync + Send>,
    /// Callback for when the user presses enter.
    on_submit: Option<Cb<dyn Fn(String) + Sync + Send>>,
    /// Whether the text should be obfuscated with '*' characters.
    password: bool,
    /// The placeholder text to display when the value is empty.
    placeholder: Option<String>,
    /// Whether the text editor should be focused when it is created.
    auto_focus: bool,
) -> Element {
    let (focused, set_focused) = use_focus(hooks);
    let (command, set_command) = use_state(hooks, false);
    let intermediate_value = use_ref_with(hooks, |_| value.clone());
    let cursor_position = use_ref_with(hooks, |_| value.len());
    let rerender = use_rerender_signal(hooks);
    {
        let mut inter = intermediate_value.lock();
        if *inter != value {
            let mut cp = cursor_position.lock();
            *cp = cp.min(value.len());
        }
        *inter = value.clone();
    }

    use_spawn(hooks, {
        to_owned![set_focused];
        move |world| {
            if auto_focus {
                set_focused(world, true);
            }
            move |world| {
                if focused {
                    set_focused(world, false);
                }
            }
        }
    });

    use_runtime_message::<messages::WindowKeyboardCharacter>(hooks, {
        to_owned![intermediate_value, on_change, cursor_position];
        move |_world, event| {
            let c = event.character.chars().next().unwrap();
            if command || !focused {
                return;
            }

            let mut value = intermediate_value.lock();
            value.insert(*cursor_position.lock(), c);
            *cursor_position.lock() += 1;
            on_change.0(value.clone());
        }
    });
    use_keyboard_input(hooks, {
        to_owned![intermediate_value, on_change, cursor_position];
        move |world, keycode, _modifiers, pressed| {
            if !focused {
                return;
            }
            if let Some(kc) = keycode {
                match kc {
                    VirtualKeyCode::LWin => {
                        #[cfg(target_os = "macos")]
                        set_command(pressed);
                    }
                    VirtualKeyCode::LControl => {
                        #[cfg(not(target_os = "macos"))]
                        set_command(pressed);
                    }
                    VirtualKeyCode::C if command && pressed => {
                        let value = intermediate_value.lock().clone();
                        ambient_guest_bridge::run_async_local(world, move || async move {
                            let _ = ambient_guest_bridge::window::set_clipboard(&value).await;
                        })
                    }
                    VirtualKeyCode::V if command && pressed => {
                        let on_change = on_change.clone();
                        let cursor_position = cursor_position.clone();
                        let intermediate_value = intermediate_value.clone();
                        ambient_guest_bridge::run_async_local(world, move || async move {
                            if let Some(paste) = ambient_guest_bridge::window::get_clipboard().await
                            {
                                let mut value = intermediate_value.lock();
                                value.insert_str(*cursor_position.lock(), &paste);
                                *cursor_position.lock() += paste.len();
                                on_change.0(value.clone());
                            }
                        })
                    }
                    VirtualKeyCode::Left => {
                        if pressed && *cursor_position.lock() > 0 {
                            *cursor_position.lock() -= 1;
                            rerender();
                        }
                    }
                    VirtualKeyCode::Right => {
                        if pressed && *cursor_position.lock() < intermediate_value.lock().len() {
                            *cursor_position.lock() += 1;
                            rerender();
                        }
                    }
                    VirtualKeyCode::Back => {
                        if pressed && *cursor_position.lock() > 0 {
                            let mut value = intermediate_value.lock();
                            value.remove(*cursor_position.lock() - 1);
                            *cursor_position.lock() -= 1;
                            on_change.0(value.clone());
                        }
                    }
                    VirtualKeyCode::Delete => {
                        if pressed && *cursor_position.lock() < intermediate_value.lock().len() {
                            let mut value = intermediate_value.lock();
                            value.remove(*cursor_position.lock());
                            on_change.0(value.clone());
                        }
                    }
                    VirtualKeyCode::Return => {
                        if pressed && !command {
                            if let Some(on_submit) = on_submit.clone() {
                                on_submit.0(intermediate_value.lock().clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    });
    let (cursor_left, cursor_right) = value.split_at(*cursor_position.lock());
    let [a, b]: [Element; 2] = [cursor_left, cursor_right]
        .iter()
        .map(|value| {
            Text.el()
                .with(
                    text(),
                    if password {
                        value.chars().map(|_| '*').collect()
                    } else {
                        value.to_string()
                    },
                )
                .with(color(), vec4(0.9, 0.9, 0.9, 1.))
        })
        .collect_vec()
        .try_into()
        .unwrap();

    with_rect(if focused {
        if !cursor_left.is_empty() {
            FlowRow::el([a, Cursor.el(), b])
        } else {
            FlowRow::el([Cursor.el(), b])
        }
    } else if value.is_empty() && !focused && placeholder.is_some() {
        FlowRow::el([Text
            .el()
            .with(text(), placeholder.unwrap())
            .with(color(), vec4(1., 1., 1., 0.2))])
    } else {
        FlowRow::el([a, b])
    })
    .with(min_width(), 3.)
    .with(min_height(), 13.)
    .with(background_color(), vec4(0., 0., 0., 0.5))
    .with(focusable(), hooks.instance_id().to_string())
    .with_clickarea()
    .on_mouse_enter(|world, _| {
        set_cursor(world, CursorIcon::Text);
    })
    .on_mouse_leave(|world, _| {
        set_cursor(world, CursorIcon::Default);
    })
    .el()
}

impl TextEditor {
    /// Create a new text editor.
    pub fn new(value: String, on_change: Cb<dyn Fn(String) + Sync + Send>) -> Self {
        Self {
            value,
            on_change,
            on_submit: None,
            password: false,
            placeholder: None,
            auto_focus: false,
        }
    }
    /// Set the `on_submit` callback.
    pub fn on_submit(mut self, on_submit: impl Fn(String) + Sync + Send + 'static) -> Self {
        self.on_submit = Some(cb(on_submit));
        self
    }
    /// Set the placeholder text.
    pub fn placeholder<T: Into<String>>(mut self, placeholder: Option<T>) -> Self {
        self.placeholder = placeholder.map(|x| x.into());
        self
    }
    /// Set whether or not the text should be hidden.
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }
    /// Focus the text box automatically when it's spawned.
    pub fn auto_focus(mut self) -> Self {
        self.auto_focus = true;
        self
    }
}

#[element_component]
fn Cursor(_hooks: &mut Hooks) -> Element {
    CursorInner::el(Instant::now())
}
#[element_component]
fn CursorInner(hooks: &mut Hooks, render_time: Instant) -> Element {
    let rerender = use_rerender_signal(hooks);
    use_frame(hooks, move |_| rerender());
    let delta = (Instant::now().duration_since(render_time).as_secs_f32() * 2.) as u32;
    if delta % 2 == 0 {
        UIBase.el().children(vec![Rectangle
            .el()
            .with(width(), 2.)
            .with(height(), 13.)
            .with(translation(), vec3(1., 0., 0.))])
    } else {
        Element::new()
    }
}

impl Editor for String {
    fn editor(self, on_change: Cb<dyn Fn(Self) + Sync + Send>, _: EditorOpts) -> Element {
        TextEditor::new(self, on_change)
            .placeholder(Some("Empty"))
            .el()
    }

    fn view(self, _opts: EditorOpts) -> Element {
        Text.el().with(text(), self)
    }
}
