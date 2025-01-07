use std::fmt::Debug;

use ambient_cb::{cb, Cb};
use ambient_element::{
    element_component, to_owned, use_state, Element, ElementComponent, ElementComponentExt, Hooks,
};
use ambient_guest_bridge::core::layout::{
    components::{align_vertical, space_between_items},
    types::Align,
};

use super::{ChangeCb, Editor, EditorOpts};
use crate::{
    button::{Button, ButtonStyle},
    default_theme::{StylesExt, STREET},
    layout::{FlowColumn, FlowRow},
    screens::{DialogScreen, ScreenContainer},
    scroll_area::{ScrollArea, ScrollAreaSizing},
    text::Text,
};

/// Delegates a type editor to edit in a new `screen`.
#[derive(Debug, Clone)]
pub struct OffscreenEditor<T> {
    /// The value to edit.
    pub value: T,
    /// Options for the editor.
    pub opts: EditorOpts,
    /// The editor to use.
    pub editor: Cb<dyn Fn(T, Option<ChangeCb<T>>, EditorOpts) -> Element + Sync + Send>,
    /// Callback for when the value is confirmed.
    pub on_confirm: Option<ChangeCb<T>>,
    /// The title of the editor.
    pub title: String,
}

impl<T: Debug + Clone + Sync + Send + 'static + Editor> ElementComponent for OffscreenEditor<T> {
    fn render(self: Box<Self>, hooks: &mut ambient_element::Hooks) -> Element {
        let Self {
            title,
            value,
            on_confirm,
            editor,
            opts,
        } = *self;

        let (screen, set_screen) = use_state(hooks, None);

        FlowRow(vec![
            ScreenContainer(screen).el(),
            Button::new("\u{fb4e} Edit", move |_| {
                set_screen(Some(
                    EditorScreen {
                        value: value.clone(),
                        title: title.clone(),
                        edit: on_confirm.is_some(),
                        on_confirm: cb({
                            to_owned![on_confirm, set_screen];
                            move |value| {
                                if let Some(on_confirm) = on_confirm.as_ref() {
                                    on_confirm(value);
                                }
                                set_screen(None);
                            }
                        }),
                        on_cancel: cb({
                            to_owned![set_screen];
                            move || {
                                set_screen(None);
                            }
                        }),
                        editor: editor.clone(),
                        opts: opts.clone(),
                    }
                    .el(),
                ));
            })
            .style(ButtonStyle::Flat)
            .el(),
        ])
        .el()
    }
}

#[element_component]
fn EditorScreen<T: Debug + Clone + Sync + Send + 'static + Editor>(
    hooks: &mut Hooks,
    value: T,
    title: String,
    on_confirm: Cb<dyn Fn(T) + Sync + Send>,
    on_cancel: Cb<dyn Fn() + Sync + Send>,
    edit: bool,
    editor: Cb<dyn Fn(T, Option<ChangeCb<T>>, EditorOpts) -> Element + Sync + Send>,
    opts: EditorOpts,
) -> Element {
    let (value, set_value) = use_state(hooks, value);
    DialogScreen(ScrollArea::el(
        ScrollAreaSizing::FitParentWidth,
        FlowColumn::el([
            Text::el(title).header_style(),
            editor(
                value.clone(),
                if edit { Some(set_value.clone()) } else { None },
                opts,
            ),
            FlowRow(vec![
                Button::new_once("Ok", move |_| on_confirm(value))
                    .style(ButtonStyle::Primary)
                    .el(),
                Button::new_once("Cancel", move |_| on_cancel())
                    .style(ButtonStyle::Flat)
                    .el(),
            ])
            .el()
            .with(align_vertical(), Align::Center)
            .with(space_between_items(), STREET),
        ])
        .with(space_between_items(), STREET),
    ))
    .el()
}
