use std::{collections::HashMap, fmt::Debug, hash::Hash, ops::Deref, sync::Arc};

use ambient_cb::{cb, Cb};
use ambient_color::Color;
use ambient_element::{
    element_component, to_owned, use_runtime_message, use_state, Element, ElementComponent,
    ElementComponentExt, Hooks,
};
use ambient_guest_bridge::core::{
    layout::{
        components::{
            fit_horizontal, fit_vertical, height, margin, min_width, padding, space_between_items,
            width,
        },
        types::Fit,
    },
    messages,
};
use ambient_shared_types::VirtualKeyCode;
use indexmap::IndexMap;
use itertools::Itertools;

use super::{Editor, EditorOpts};
use crate::{
    button::{Button, ButtonStyle},
    default_theme::{
        StylesExt, COLLECTION_ADD_ICON, COLLECTION_DELETE_ICON, MOVE_DOWN_ICON, MOVE_UP_ICON,
        STREET,
    },
    dropdown::Dropdown,
    layout::{FlowColumn, FlowRow},
    use_focus, use_keyboard_input, UIBase, UIExt,
};
use glam::{vec4, Vec4};

#[element_component]
/// An editor for a list of items that implement [Editor]; each item can be edited, moved up or down, or deleted.
pub fn ListEditor<T: Editor + std::fmt::Debug + Clone + Default + Sync + Send + 'static>(
    _: &mut Hooks,
    /// The list of items to edit.
    value: Vec<T>,
    /// A callback that is called when the list of items is changed.
    on_change: Option<Cb<dyn Fn(Vec<T>) + Sync + Send>>,
) -> Element {
    if let Some(on_change) = on_change {
        let button_size = 20.;
        FlowColumn::el([
            FlowColumn(
                value
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        FlowRow(vec![
                            Button::new(COLLECTION_DELETE_ICON, {
                                to_owned![on_change, value];
                                move |_| {
                                    let mut value = value.clone();
                                    value.remove(i);
                                    on_change.0(value);
                                }
                            })
                            .style(ButtonStyle::Flat)
                            .el()
                            .with(min_width(), button_size),
                            if i > 0 {
                                Button::new(MOVE_UP_ICON, {
                                    to_owned![on_change, value];
                                    move |_| {
                                        let mut value = value.clone();
                                        value.swap(i, i - 1);
                                        on_change.0(value);
                                    }
                                })
                                .style(ButtonStyle::Flat)
                                .el()
                                .with(min_width(), button_size)
                            } else {
                                UIBase.el().with(width(), button_size).with(height(), 1.)
                            },
                            if i < value.len() - 1 {
                                Button::new(MOVE_DOWN_ICON, {
                                    to_owned![on_change, value];
                                    move |_| {
                                        let mut value = value.clone();
                                        value.swap(i, i + 1);
                                        on_change.0(value);
                                    }
                                })
                                .style(ButtonStyle::Flat)
                                .el()
                                .with(min_width(), button_size)
                            } else {
                                UIBase.el().with(width(), button_size).with(height(), 1.)
                            },
                            T::edit_or_view(
                                item.clone(),
                                Some(cb({
                                    to_owned![value, on_change];
                                    move |item| {
                                        let mut value = value.clone();
                                        value[i] = item;
                                        on_change.0(value);
                                    }
                                })),
                                Default::default(),
                            ),
                        ])
                        .el()
                    })
                    .collect(),
            )
            .el(),
            Button::new(COLLECTION_ADD_ICON, {
                to_owned![on_change];
                move |_| {
                    let mut value = value.clone();
                    value.push(T::default());
                    on_change.0(value);
                }
            })
            .style(ButtonStyle::Flat)
            .el(),
        ])
    } else {
        unimplemented!()
    }
}

impl<T: Editor + std::fmt::Debug + Clone + Default + Sync + Send + 'static> Editor for Vec<T> {
    fn editor(self, on_change: Cb<dyn Fn(Self) + Sync + Send>, _: EditorOpts) -> Element {
        ListEditor {
            value: self,
            on_change: Some(on_change),
        }
        .el()
    }
    fn view(self, _: EditorOpts) -> Element {
        ListEditor {
            value: self,
            on_change: None,
        }
        .el()
    }
}

#[derive(Debug, Clone)]
/// A [MinimalListEditorWithItemEditor] that uses the default editor for the items.
pub struct MinimalListEditor<T: Editor + std::fmt::Debug + Clone + Default + Sync + Send + 'static>
{
    /// The list of items to edit.
    pub value: Vec<T>,
    /// A callback that is called when the list of items is changed.
    pub on_change: Option<Cb<dyn Fn(Vec<T>) + Sync + Send>>,
    /// Options for the item editor.
    pub item_opts: EditorOpts,
    /// Preset items that can be added to the list.
    pub add_presets: Option<Vec<T>>,
    /// The title for the add button.
    pub add_title: String,
}
impl<T: Editor + std::fmt::Debug + Clone + Default + Sync + Send + 'static> ElementComponent
    for MinimalListEditor<T>
{
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        MinimalListEditorWithItemEditor {
            value: self.value,
            on_change: self.on_change,
            item_opts: self.item_opts,
            add_presets: self.add_presets,
            add_title: self.add_title,
            item_editor: cb(T::edit_or_view),
        }
        .el()
    }
}

#[allow(clippy::type_complexity)]
#[derive(Debug, Clone)]
/// A configurable list editor that allows you to specify the editor for the items, and to provide other preset options.
pub struct MinimalListEditorWithItemEditor<
    T: std::fmt::Debug + Clone + Default + Sync + Send + 'static,
> {
    /// The list of items to edit.
    pub value: Vec<T>,
    /// A callback that is called when the list of items is changed.
    pub on_change: Option<Cb<dyn Fn(Vec<T>) + Sync + Send>>,
    /// Options for the item editor.
    pub item_opts: EditorOpts,
    /// Preset items that can be added to the list.
    pub add_presets: Option<Vec<T>>,
    /// The title of the add button.
    pub add_title: String,
    /// The editor for the items.
    pub item_editor:
        Cb<dyn Fn(T, Option<Cb<dyn Fn(T) + Sync + Send>>, EditorOpts) -> Element + Sync + Send>,
}
impl<T: std::fmt::Debug + Clone + Default + Sync + Send + 'static> ElementComponent
    for MinimalListEditorWithItemEditor<T>
{
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self {
            value,
            on_change,
            item_opts,
            add_presets,
            add_title,
            item_editor,
        } = *self;
        let (add_action, set_add_action) = use_state(hooks, false);
        let has_on_change = on_change.is_some();
        use_runtime_message::<messages::WindowMouseInput>(hooks, {
            let set_add_action = set_add_action.clone();
            move |_world, event| {
                let pressed = event.pressed;
                if pressed && has_on_change {
                    set_add_action(false);
                }
            }
        });
        FlowColumn::el([
            FlowColumn(
                value
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        MinimalListEditorItem {
                            value: item.clone(),
                            on_change: on_change.clone().map(
                                |on_change| -> Cb<dyn Fn(T) + Sync + Send> {
                                    cb({
                                        to_owned![value, on_change];
                                        move |item| {
                                            let mut value = value.clone();
                                            value[i] = item;
                                            on_change.0(value);
                                        }
                                    })
                                },
                            ),
                            on_delete: on_change.clone().map(
                                |on_change| -> Cb<dyn Fn() + Sync + Send> {
                                    cb({
                                        to_owned![value, on_change];
                                        move || {
                                            let mut value = value.clone();
                                            value.remove(i);
                                            on_change.0(value);
                                        }
                                    })
                                },
                            ),
                            item_opts: item_opts.clone(),
                            item_editor: item_editor.clone(),
                        }
                        .el()
                    })
                    .collect(),
            )
            .el()
            .with(fit_horizontal(), Fit::Parent),
            if let Some(on_change) = on_change {
                if let Some(add_presets) = add_presets {
                    Dropdown {
                        content: Button::new(add_title, {
                            to_owned![set_add_action];
                            move |_| {
                                set_add_action(true);
                            }
                        })
                        .style(ButtonStyle::Flat)
                        .el(),
                        dropdown: FlowColumn(
                            add_presets
                                .into_iter()
                                .map(move |item| {
                                    item_editor.0(item.clone(), None, Default::default())
                                        .with_clickarea()
                                        .on_mouse_down({
                                            to_owned![value, on_change];
                                            move |_, _, _| {
                                                let mut value = value.clone();
                                                value.push(item.clone());
                                                on_change.0(value);
                                            }
                                        })
                                        .el()
                                        .with_margin_even(STREET)
                                })
                                .collect(),
                        )
                        .el()
                        .with_background(Color::rgba(0.05, 0.05, 0.05, 1.).into())
                        .with(fit_horizontal(), Fit::None)
                        .with(width(), 400.),
                        show: add_action,
                    }
                    .el()
                    .with(margin(), Vec4::X * STREET)
                } else {
                    Button::new(add_title, {
                        to_owned![value, on_change];
                        move |_| {
                            let mut value = value.clone();
                            value.push(T::default());
                            on_change.0(value);
                        }
                    })
                    .style(ButtonStyle::Flat)
                    .el()
                }
            } else {
                Element::new()
            },
        ])
    }
}

#[allow(clippy::type_complexity)]
#[derive(Debug, Clone)]
/// A single item in a list editor
pub struct MinimalListEditorItem<T: std::fmt::Debug + Clone + Default + Sync + Send + 'static> {
    /// The value of the item.
    pub value: T,
    /// A callback that is called when the item is changed.
    pub on_change: Option<Cb<dyn Fn(T) + Sync + Send>>,
    /// A callback that is called when the item is deleted.
    pub on_delete: Option<Cb<dyn Fn() + Sync + Send>>,
    /// Options for the item editor.
    pub item_opts: EditorOpts,
    /// The editor for the item.
    pub item_editor:
        Cb<dyn Fn(T, Option<Cb<dyn Fn(T) + Sync + Send>>, EditorOpts) -> Element + Sync + Send>,
}
impl<T: std::fmt::Debug + Clone + Default + Sync + Send + 'static> ElementComponent
    for MinimalListEditorItem<T>
{
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self {
            value,
            on_change,
            on_delete,
            item_opts,
            item_editor,
        } = *self;
        let (focused, set_focused) = use_focus(hooks);
        use_keyboard_input(hooks, move |_world, keycode, _modifiers, pressed| {
            if !focused || !pressed {
                return;
            }
            if let Some(on_delete) = &on_delete {
                if let Some(keycode) = keycode {
                    if keycode == VirtualKeyCode::Back || keycode == VirtualKeyCode::Delete {
                        on_delete.0();
                    }
                }
            }
        });
        FlowRow(vec![
            UIBase
                .el()
                .with(width(), 5.)
                .with(fit_vertical(), Fit::Parent)
                .with_background(
                    if focused {
                        Color::rgba(0.0, 1., 0., 1.)
                    } else {
                        Color::rgba(0.5, 0.5, 0.5, 1.)
                    }
                    .into(),
                )
                .with(margin(), vec4(0., 5., 0., 0.)),
            item_editor.0(value, on_change, item_opts).with(fit_horizontal(), Fit::Parent),
        ])
        .el()
        .with_clickarea()
        .on_mouse_down(move |world, _, _| {
            set_focused(world, true);
        })
        .el()
        .with(padding(), vec4(STREET, 0., STREET, 0.))
        .with(fit_horizontal(), Fit::Parent)
    }
}

#[allow(clippy::type_complexity)]
#[derive(Debug, Clone)]
/// An editor for a [HashMap]. The key and value types must implement [Editor].
///
/// The pairs are sorted by key.
pub struct KeyValueEditor<
    K: Editor
        + std::fmt::Debug
        + Clone
        + Default
        + Hash
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Sync
        + Send
        + 'static,
    V: Editor + std::fmt::Debug + Clone + Default + Sync + Send + 'static,
> {
    /// The value to edit.
    pub value: HashMap<K, V>,
    /// A callback that is called when the value is changed.
    pub on_change: Option<Cb<dyn Fn(HashMap<K, V>) + Sync + Send>>,
}
impl<
        K: Editor
            + std::fmt::Debug
            + Clone
            + Default
            + Hash
            + PartialEq
            + Eq
            + PartialOrd
            + Ord
            + Sync
            + Send
            + 'static,
        V: Editor + std::fmt::Debug + Clone + Default + Sync + Send + 'static,
    > ElementComponent for KeyValueEditor<K, V>
{
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        let Self { value, on_change } = *self;
        FlowColumn::el([
            FlowColumn(
                value
                    .clone()
                    .into_iter()
                    .sorted_by_key(|(key, _)| key.clone())
                    .map({
                        to_owned![value, on_change];
                        move |(key, item)| {
                            FlowRow(vec![
                                K::edit_or_view(
                                    key.clone(),
                                    on_change.clone().map(
                                        |on_change| -> Cb<dyn Fn(K) + Sync + Send> {
                                            cb({
                                                to_owned![key, on_change, value];
                                                move |new_key| {
                                                    let mut value = value.clone();
                                                    let item = value.remove(&key).unwrap();
                                                    value.insert(new_key, item);
                                                    on_change.0(value);
                                                }
                                            })
                                        },
                                    ),
                                    Default::default(),
                                ),
                                V::edit_or_view(
                                    item,
                                    on_change.clone().map(
                                        |on_change| -> Cb<dyn Fn(V) + Sync + Send> {
                                            cb({
                                                to_owned![value, on_change];
                                                move |item| {
                                                    let mut value = value.clone();
                                                    value.insert(key.clone(), item);
                                                    on_change.0(value);
                                                }
                                            })
                                        },
                                    ),
                                    Default::default(),
                                ),
                            ])
                            .el()
                        }
                    })
                    .collect(),
            )
            .el(),
            if let Some(on_change) = on_change {
                Button::new("Add", move |_| {
                    let mut value = value.clone();
                    value.insert(Default::default(), Default::default());
                    on_change(value);
                })
                .el()
            } else {
                Element::new()
            },
        ])
    }
}
impl<
        K: Editor
            + std::fmt::Debug
            + Clone
            + Default
            + Hash
            + PartialEq
            + Eq
            + PartialOrd
            + Ord
            + Sync
            + Send
            + 'static,
        V: Editor + std::fmt::Debug + Clone + Default + Sync + Send + 'static,
    > Editor for HashMap<K, V>
{
    fn editor(self, on_change: Cb<dyn Fn(Self) + Sync + Send>, _: EditorOpts) -> Element {
        KeyValueEditor {
            value: self,
            on_change: Some(on_change),
        }
        .el()
    }

    fn view(self, _: EditorOpts) -> Element {
        KeyValueEditor {
            value: self,
            on_change: None,
        }
        .el()
    }
}

#[derive(Debug, Clone)]
/// An editor for a [IndexMap]. The key and value types must implement [Editor].
pub struct IndexMapEditor<K, V> {
    /// The value to edit.
    value: Arc<IndexMap<K, V>>,
    /// A callback that is called when the value is changed.
    on_change: Cb<dyn Fn(IndexMap<K, V>) + Send + Sync>,

    use_row_instead_of_column: bool,
}

impl<K, V> IndexMapEditor<K, V> {
    /// Create a new [IndexMapEditor] with the given value and callback.
    pub fn new(
        value: IndexMap<K, V>,
        on_change: Cb<dyn Fn(IndexMap<K, V>) + Send + Sync>,
        use_row_instead_of_column: bool,
    ) -> Self {
        Self {
            value: Arc::new(value),
            on_change,
            use_row_instead_of_column,
        }
    }
}
impl<K, V> ElementComponent for IndexMapEditor<K, V>
where
    K: Hash + Eq + Send + Sync + Debug + 'static + Clone + Editor + Default,
    V: Send + Sync + Debug + 'static + Clone + Editor + Default,
{
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        let fields = self.value.iter().map(|(key, value)| {
            IndexMapEntryPart {
                key: key.clone(),
                value: value.clone(),
                parent: self.value.clone(),
                on_change: self.on_change.clone(),
            }
            .el()
        });

        let map = self.value.clone();
        let on_change = self.on_change.clone();

        let add = Button::new(COLLECTION_ADD_ICON, move |_w| {
            let mut map = map.deref().clone();
            let key = Default::default();
            map.remove(&key);
            map.insert(key, Default::default());
            on_change(map)
        })
        .style(ButtonStyle::Flat)
        .el();

        let fields = fields.chain([add]).collect_vec();
        if self.use_row_instead_of_column {
            FlowRow(fields).el()
        } else {
            FlowColumn(fields).el()
        }
        .with(space_between_items(), STREET)
    }
}

impl<K, V> Editor for IndexMap<K, V>
where
    K: std::hash::Hash + Eq + Send + Sync + Debug + 'static + Clone + Editor + Default,
    V: Send + Sync + Debug + 'static + Clone + Editor + Default,
{
    fn editor(self, on_change: Cb<dyn Fn(Self) + Send + Sync>, _opts: EditorOpts) -> Element {
        IndexMapEditor::new(self, on_change, false).el()
    }

    fn view(self, opts: EditorOpts) -> Element {
        let fields = self
            .into_iter()
            .map(|(k, v)| FlowColumn(vec![K::view(k, opts.clone()), V::view(v, opts.clone())]).el())
            .collect_vec();
        FlowColumn(fields).el().with(space_between_items(), STREET)
    }
}

#[derive(Debug, Clone)]
struct IndexMapEntryPart<K, V> {
    key: K,
    value: V,
    parent: Arc<IndexMap<K, V>>,
    on_change: Cb<dyn Fn(IndexMap<K, V>) + Send + Sync>,
}

impl<K, V> ElementComponent for IndexMapEntryPart<K, V>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static + Editor,
    V: Clone + Debug + Editor + Send + Sync + 'static,
{
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        let Self {
            key,
            value,
            on_change,
            parent,
        } = *self;

        let key_editor = {
            let parent = parent.clone();
            let on_change = on_change.clone();
            let old_key = key.clone();
            K::edit_or_view(
                key.clone(),
                Some(cb(move |key| {
                    let mut map = parent.deref().clone();
                    let val = map.remove(&old_key).expect("Missing key in map");
                    map.insert(key, val);
                    on_change(map);
                })),
                Default::default(),
            )
        };

        let value_editor = {
            let key = key.clone();
            let on_change = on_change.clone();
            let parent = parent.clone();
            V::edit_or_view(
                value,
                Some(cb(move |value| {
                    let mut map = parent.deref().clone();
                    map.insert(key.clone(), value);

                    on_change(map)
                })),
                Default::default(),
            )
        };

        let discard = {
            let map = parent;
            Button::new(COLLECTION_DELETE_ICON, move |_| {
                let mut map = map.deref().clone();
                map.remove(&key)
                    .expect("Can not remove non existent element");
                on_change(map)
            })
            .style(ButtonStyle::Flat)
            .el()
        };

        FlowColumn(vec![
            FlowRow(vec![discard, key_editor])
                .el()
                .with(space_between_items(), STREET),
            value_editor,
        ])
        .el()
        .panel()
        .with(space_between_items(), STREET)
        .with_padding_even(STREET)
    }
}
