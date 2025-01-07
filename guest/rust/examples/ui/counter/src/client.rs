use ambient_api::{core::layout::components::space_between_items, element::use_state, prelude::*};

pub mod packages;

#[main]
pub fn main() {
    App.el().spawn_interactive();
}

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let (count, set_count) = use_state(hooks, 0);
    FlowColumn::el([
        Text::el(format!("We've counted to {count} now")),
        Button::new("Increase", move |_| set_count(count + 1)).el(),
    ])
    .with_padding_even(STREET)
    .with(space_between_items(), STREET)
}
