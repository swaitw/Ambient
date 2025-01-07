use ambient_api::{
    core::layout::components::{min_width, space_between_items, width},
    element::use_state,
    prelude::*,
};
use indexmap::IndexMap;

pub mod packages;

#[main]
pub fn main() {
    App.el().spawn_interactive();
}

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let (text, set_text) = use_state(hooks, "Enter some text".to_string());
    let (float, set_float) = use_state(hooks, 0.0);
    let (vector3, set_vector3) = use_state(hooks, Vec3::ZERO);
    let (index_map, set_index_map) = use_state(
        hooks,
        vec![("First".to_string(), "Second".to_string())]
            .into_iter()
            .collect::<IndexMap<String, String>>(),
    );
    let (list, set_list) = use_state(hooks, vec!["First".to_string(), "Second".to_string()]);
    let (minimal_list, set_minimal_list) =
        use_state(hooks, vec!["First".to_string(), "Second".to_string()]);
    let row = |name, editor| FlowRow::el(vec![Text::el(name).with(min_width(), 110.), editor]);
    FlowColumn::el([
        row("TextEditor", TextEditor::new(text, set_text).el()),
        row(
            "F32Input",
            F32Input {
                value: float,
                on_change: set_float,
            }
            .el(),
        ),
        row(
            "DropDownSelect",
            DropdownSelect {
                content: Text::el("Select"),
                on_select: cb(|_| {}),
                items: vec![Text::el("First"), Text::el("Second")],
                inline: false,
            }
            .el(),
        ),
        row(
            "Vec3",
            Vec3::editor(vector3, set_vector3, Default::default()),
        ),
        row(
            "IndexMap",
            IndexMap::editor(index_map, set_index_map, Default::default()),
        ),
        row(
            "ListEditor",
            ListEditor {
                value: list,
                on_change: Some(set_list),
            }
            .el(),
        ),
        row(
            "MinimalListEditor",
            MinimalListEditor {
                value: minimal_list,
                on_change: Some(set_minimal_list),
                item_opts: Default::default(),
                add_presets: None,
                add_title: "Add".to_string(),
            }
            .el(),
        ),
    ])
    .with(width(), 200.)
    .with(space_between_items(), STREET)
    .with_padding_even(STREET)
}
