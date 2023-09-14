use std::{collections::HashSet, fmt};

use ambient_api::{
    core::{
        package::{
            self,
            components::{description, id, is_package},
            concepts::Package as PackageConcept,
        },
        rect::components::background_color,
        text::{components::font_style, types::FontStyle},
    },
    element::{use_effect, use_module_message, use_query, use_spawn, use_state},
    prelude::*,
    ui::ImageFromUrl,
};

use crate::{
    packages::this::{
        assets,
        messages::{
            PackageLoad, PackageLoadShow, PackageRemoteRequest, PackageRemoteResponse,
            PackageSetEnabled, PackageShow,
        },
    },
    shared::PackageJson,
};

use super::use_hotkey_toggle;

#[element_component]
pub fn PackageManager(hooks: &mut Hooks) -> Element {
    let (visible, set_visible) = use_hotkey_toggle(hooks, VirtualKeyCode::F4);
    use_editor_menu_bar(hooks, "Package Manager".to_string(), {
        let set_visible = set_visible.clone();
        move || set_visible(!visible)
    });

    Window::el(
        "Package Manager".to_string(),
        visible,
        Some(cb(move || set_visible(false))),
        PackageManagerInner::el(),
    )
}

#[element_component]
fn PackageManagerInner(hooks: &mut Hooks) -> Element {
    #[derive(PartialEq, Default, Clone, Debug)]
    enum ListTab {
        #[default]
        Local,
        Remote,
    }
    impl fmt::Display for ListTab {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    ListTab::Local => "Local",
                    ListTab::Remote => "Remote",
                }
            )
        }
    }

    Tabs::new()
        .with_tab(ListTab::Local, || PackagesLocal::el())
        .with_tab(ListTab::Remote, || PackagesRemote::el())
        .el()
        .with(space_between_items(), 4.0)
        .with_margin_even(STREET)
}

#[element_component]
fn PackagesLocal(hooks: &mut Hooks) -> Element {
    let packages = use_query(hooks, PackageConcept::as_query());

    let display_packages: Vec<_> = packages
        .into_iter()
        .map(|(id, package)| {
            let description = entity::get_component(id, description());

            DisplayPackage {
                source: DisplayPackageSource::Local {
                    id,
                    enabled: package.enabled,
                },
                name: package.name,
                version: package.version,
                authors: package.authors,
                description,
            }
        })
        .collect();

    PackageList::el(display_packages)
}

#[element_component]
fn PackagesRemote(hooks: &mut Hooks) -> Element {
    let loaded_packages = use_query(hooks, (is_package(), id()));

    #[derive(Clone, Debug)]
    enum PackagesState {
        Loading,
        Loaded(Vec<PackageJson>),
        Error(String),
    }
    let (remote_packages, set_remote_packages) = use_state(hooks, PackagesState::Loading);

    use_effect(hooks, (), move |_, _| {
        PackageRemoteRequest::default().send_server_reliable();
        |_| {}
    });

    use_module_message::<PackageRemoteResponse>(hooks, move |_, ctx, msg| {
        if !ctx.server() {
            return;
        }

        if let Some(error) = &msg.error {
            set_remote_packages(PackagesState::Error(error.to_string()));
            return;
        }

        let packages_json = msg
            .packages
            .iter()
            .map(|p| serde_json::from_str::<PackageJson>(p))
            .collect::<Result<Vec<_>, _>>();

        match packages_json {
            Ok(packages_json) => set_remote_packages(PackagesState::Loaded(packages_json)),
            Err(error) => {
                set_remote_packages(PackagesState::Error(format!(
                    "Failed to parse packages: {}",
                    error
                )));
                return;
            }
        };
    });

    let loaded_package_ids: HashSet<String> =
        HashSet::from_iter(loaded_packages.into_iter().map(|(_, (_, id))| id));

    FlowColumn::el([
        match remote_packages {
            PackagesState::Loading => Text::el("Loading..."),
            PackagesState::Loaded(remote_packages) => PackageList::el(
                remote_packages
                    .into_iter()
                    .filter(|package| !loaded_package_ids.contains(&package.id))
                    .map(|package| DisplayPackage {
                        source: DisplayPackageSource::Remote {
                            url: package.url.clone(),
                        },
                        name: package.name,
                        version: package.version,
                        authors: package.authors,
                        description: package.description,
                    })
                    .collect(),
            ),
            PackagesState::Error(error) => Text::el(error),
        },
        Button::new("Load package from URL", |_| {
            PackageLoadShow.send_local(crate::packages::this::entity())
        })
        .el(),
    ])
    .with(space_between_items(), 8.0)
}

#[derive(Clone, Debug)]
struct DisplayPackage {
    source: DisplayPackageSource,
    name: String,
    version: String,
    authors: Vec<String>,
    description: Option<String>,
}

#[derive(Clone, Debug)]
enum DisplayPackageSource {
    Local { id: EntityId, enabled: bool },
    Remote { url: String },
}

#[element_component]
fn PackageList(_hooks: &mut Hooks, packages: Vec<DisplayPackage>) -> Element {
    let mut packages = packages;
    packages.sort_by_key(|package| package.name.clone());

    FlowColumn::el(packages.into_iter().map(|package| Package::el(package)))
        .with(space_between_items(), 8.0)
        .with(min_width(), 400.0)
}

#[element_component]
fn Package(_hooks: &mut Hooks, package: DisplayPackage) -> Element {
    fn button(text: impl Into<String>, action: impl Fn() + Send + Sync + 'static) -> Element {
        Button::new(text.into(), move |_| action())
            .style(ButtonStyle::Inline)
            .el()
    }

    with_rect(FlowRow::el([
        // Image (ideally, this would be the package icon)
        ImageFromUrl {
            url: assets::url("construction.png"),
        }
        .el()
        .with(width(), 64.0)
        .with(height(), 64.0),
        // Contents
        FlowColumn::el([
            // Header
            FlowRow::el([
                Text::el(package.name).with(font_style(), FontStyle::Bold),
                Text::el(package.version),
                Text::el("by"),
                Text::el(if package.authors.is_empty() {
                    "No authors specified".to_string()
                } else {
                    package.authors.join(", ")
                })
                .with(font_style(), FontStyle::Italic),
            ])
            .with(space_between_items(), 4.0),
            // Description
            Text::el(package.description.as_deref().unwrap_or("No description")),
            // Buttons
            match &package.source {
                DisplayPackageSource::Local { id, enabled } => {
                    let id = *id;
                    let enabled = *enabled;
                    FlowRow::el([
                        button(if enabled { "Disable" } else { "Enable" }, move || {
                            PackageSetEnabled {
                                id,
                                enabled: !enabled,
                            }
                            .send_server_reliable();
                        }),
                        button("View", move || {
                            PackageShow { id }.send_local(crate::packages::this::entity())
                        }),
                    ])
                    .with(space_between_items(), 8.0)
                }
                DisplayPackageSource::Remote { url } => {
                    let url = url.to_string();
                    button("Load", move || {
                        PackageLoad { url: url.clone() }.send_server_reliable();
                    })
                }
            },
        ])
        .with(space_between_items(), 4.0),
    ]))
    .with(space_between_items(), 8.0)
    .with(background_color(), vec4(0., 0., 0., 0.5))
    .with(fit_horizontal(), Fit::Parent)
    .with_padding_even(8.0)
}

// TODO: is there a way to share this?
fn use_editor_menu_bar(
    hooks: &mut Hooks,
    name: String,
    on_click: impl Fn() + Send + Sync + 'static,
) {
    use crate::packages::editor_schema::messages::{
        EditorLoad, EditorMenuBarAdd, EditorMenuBarClick,
    };

    let add = cb({
        let name = name.clone();
        move || EditorMenuBarAdd { name: name.clone() }.send_local_broadcast(false)
    });

    use_module_message::<EditorLoad>(hooks, {
        let add = add.clone();
        move |_, _, _| {
            add();
        }
    });

    use_spawn(hooks, move |_| {
        add();
        |_| {}
    });

    use_module_message::<EditorMenuBarClick>(hooks, move |_, _, message| {
        if message.name == name {
            on_click();
        }
    });
}
