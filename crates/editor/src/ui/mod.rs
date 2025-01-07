use std::{collections::HashMap, fmt::Debug, io::Cursor};

mod build_mode;
pub mod entity_editor;
mod terrain_mode;

use ambient_core::{game_mode, runtime, transform::translation, GameMode};
use ambient_ecs::{Entity, EntityId};
use ambient_element::{
    consume_context, element_component, provide_context, use_effect, use_interval, use_state,
    Element, ElementComponent, ElementComponentExt, Group, Hooks, Setter,
};
use ambient_intent::{rpc_redo, rpc_undo_head, IntentHistoryVisualizer};
use ambient_native_std::{cb, color::Color, Cb};
use ambient_naturals::{get_default_natural_layers, natural_layers, NaturalsPreset};
use ambient_network::{
    client::ClientState,
    hooks::{use_remote_persisted_resource, use_remote_player_component},
    log_network_result,
    rpc::{rpc_fork_instance, rpc_get_instances_info, rpc_join_instance, RpcForkInstance},
    server::MAIN_INSTANCE_ID,
    unwrap_log_network_err,
};
use ambient_physics::make_physics_static;
use ambient_shared_types::{ModifiersState, VirtualKeyCode};
use ambient_terrain::{
    brushes::{
        Brush, BrushShape, BrushSize, BrushSmoothness, BrushStrength, HydraulicErosionConfig,
    },
    terrain_material_def,
};
use ambient_ui_native::{
    command_modifier, height,
    layout::{docking, space_between_items, width, Borders, Docking},
    margin, use_window_logical_resolution, Button, FlowColumn, FlowRow, FontAwesomeIcon, Hotkey,
    Rectangle, ScreenContainer, ScrollArea, ScrollAreaSizing, Separator, StylesExt, Text, UIExt,
    WindowSized, STREET,
};
use build_mode::*;
use glam::{vec3, Vec3};
use image::{DynamicImage, ImageOutputFormat, RgbImage};
use itertools::Itertools;
use terrain_mode::*;

use crate::{selection, Selection};
use serde::{de::DeserializeOwned, Serialize};

pub fn use_player_selection(hooks: &mut Hooks) -> (Selection, Setter<Selection>) {
    use_remote_player_component(hooks, selection())
}

impl EditorPrefs {
    pub fn snap(self, pos: Vec3) -> Vec3 {
        match self.snap {
            None => pos,
            Some(snap) => (pos / snap).round() * snap,
        }
    }
}

#[derive(Default, Copy, Debug, Clone, PartialEq)]
/// Saves the options for the build mode and other editors
struct EditorPrefs {
    pub use_global_coordinates: bool,
    pub snap: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorMode {
    Experience,
    Terrain,
    Build,
    Atmosphere,
    NaturalLayers,
    TerrainMaterial,
}

#[derive(Debug, Clone)]
pub struct EditorSettings {
    pub debug_mode: bool,
    pub debug_intents: bool,
    pub show_hud: bool,
}

#[derive(Debug, Clone)]
pub struct EditingEntityContext(pub EntityId);

const PLAY_INSTANCE_ID: &str = "play";

#[element_component]
pub fn EditorUI(hooks: &mut Hooks) -> Element {
    let (editor_mode, set_editor_mode) = use_state(hooks, EditorMode::Build);

    let (client_state, _) = consume_context::<ClientState>(hooks).unwrap();
    let (hide_ui, set_hide_ui) = use_state(hooks, false);
    let (user_settings, _) = consume_context::<EditorSettings>(hooks).unwrap();
    let (screen, _set_screen) = use_state(hooks, None);

    provide_context(hooks, EditorPrefs::default);

    provide_context(hooks, || Brush::Raise);
    provide_context(hooks, || 0u32);
    provide_context(hooks, || BrushSize::SMALL);
    provide_context(hooks, || BrushStrength::MEDIUM);
    provide_context(hooks, || BrushShape::Circle);
    provide_context(hooks, || BrushSmoothness(1.));
    provide_context(hooks, HydraulicErosionConfig::default);

    use_effect(hooks, editor_mode, {
        let client_state = client_state.clone();
        move |world, _| {
            world.resource(runtime()).spawn(async move {
                if editor_mode == EditorMode::Experience {
                    let id = unwrap_log_network_err!(
                        client_state
                            .rpc(
                                rpc_fork_instance,
                                RpcForkInstance {
                                    resources: Entity::new().with(make_physics_static(), false),
                                    synced_res: Entity::new().with(game_mode(), GameMode::Play),
                                    id: Some(PLAY_INSTANCE_ID.to_string())
                                }
                            )
                            .await
                    );
                    log_network_result!(client_state.rpc(rpc_join_instance, id).await);
                } else {
                    log_network_result!(
                        client_state
                            .rpc(rpc_join_instance, MAIN_INSTANCE_ID.to_string())
                            .await
                    );
                }
            });
            |_| {}
        }
    });

    if hide_ui {
        return Hotkey::new(
            VirtualKeyCode::Escape,
            closure!(clone set_hide_ui, |_| set_hide_ui(false)),
            EditorPlayerInputHandler.el(),
        )
        .el();
    }

    Group(vec![
        Crosshair.el(),
        WindowSized(vec![
            ScreenContainer(screen).el(),
            FlowColumn::el([FlowRow::el([
                Button::new(
                    FontAwesomeIcon::el(0xf21c, true),
                    closure!(clone set_editor_mode, |_| set_editor_mode(EditorMode::Experience)),
                )
                .hotkey(VirtualKeyCode::F1)
                .toggled(editor_mode == EditorMode::Experience)
                .tooltip("Experience")
                .el(),
                Button::new(
                    FontAwesomeIcon::el(0xf6e3, true),
                    closure!(clone set_editor_mode, |_| {
                        set_editor_mode(EditorMode::Build);

                    }),
                )
                .hotkey(VirtualKeyCode::F2)
                .toggled(editor_mode == EditorMode::Build)
                .tooltip("Build")
                .el(),
                Button::new(FontAwesomeIcon::el(0xe52f, true), closure!(clone set_editor_mode, |_| set_editor_mode(EditorMode::Terrain)))
                    .hotkey(VirtualKeyCode::F3)
                    .toggled(editor_mode == EditorMode::Terrain)
                    .tooltip("Terrain")
                    .el(),
                Button::new(
                    FontAwesomeIcon::el(0xf73c, true),
                    closure!(clone set_editor_mode, |_| set_editor_mode(EditorMode::Atmosphere)),
                )
                .hotkey(VirtualKeyCode::F5)
                .toggled(editor_mode == EditorMode::Atmosphere)
                .tooltip("Atmosphere")
                .el(),
                Button::new(
                    FontAwesomeIcon::el(0xf1bb, true),
                    closure!(clone set_editor_mode, |_| set_editor_mode(EditorMode::NaturalLayers)),
                )
                .hotkey(VirtualKeyCode::F7)
                .toggled(editor_mode == EditorMode::NaturalLayers)
                .tooltip("Biomes")
                .el(),
                Button::new(
                    FontAwesomeIcon::el(0xf06c, true),
                    closure!(clone set_editor_mode, |_| set_editor_mode(EditorMode::TerrainMaterial)),
                )
                .hotkey(VirtualKeyCode::F8)
                .toggled(editor_mode == EditorMode::TerrainMaterial)
                .tooltip("Ground materials")
                .el(),
                Separator { vertical: true }.el(),
                Button::new(FontAwesomeIcon::el(0xf815, true), closure!(clone set_hide_ui, |_| set_hide_ui(true)))
                    .hotkey(VirtualKeyCode::P)
                    .hotkey_modifier(command_modifier())
                    .tooltip("Hide UI")
                    .el(),
                // UploadThumbnailButton.el(),
                Button::new_async(FontAwesomeIcon::el(0xf2ea, true), {
                    let client_state = client_state.clone();
                    move || {
                        let client_state = client_state.clone();
                        async move {
                            client_state.rpc(rpc_undo_head, ()).await.ok();
                        }
                    }
                })
                .hotkey(VirtualKeyCode::Z)
                .hotkey_modifier(command_modifier())
                .tooltip("Undo")
                .el(),
                Button::new_async(FontAwesomeIcon::el(0xf2f9, true), move || {
                    let client_state = client_state.clone();
                    async move {
                        client_state.rpc(rpc_redo, ()).await.ok();
                    }
                })
                .hotkey(VirtualKeyCode::Z)
                .hotkey_modifier(command_modifier() | ModifiersState::SHIFT)
                .tooltip("Redo")
                .el(),
                ServerInstancesInfo.el(),
            ])
            .floating_panel()
            .keyboard()
            .with(margin(), Borders::even(STREET).set_bottom(0.).into())]),
            if user_settings.debug_intents {
                IntentHistoryVisualizer.el().with(margin(), Borders::even(STREET).into()).with(docking(), Docking::Top)
            } else {
                Element::new()
            },
            match editor_mode {
                EditorMode::Experience => EditorExperienceMode.el(),
                EditorMode::Terrain => EditorTerrainMode.el(),
                EditorMode::Build => EditorBuildMode.el(),
                EditorMode::Atmosphere => EditorAtmosphereMode.el(),
                EditorMode::NaturalLayers => NaturalLayersEditor.el().with(docking(), Docking::Left).with(width(), 500.),
                EditorMode::TerrainMaterial => TerrainMaterialEditor.el().with(docking(), Docking::Left).with(width(), 500.),
            },
        ])
        .el(),
    ])
    .el()
}

#[element_component]
fn ServerInstancesInfo(hooks: &mut Hooks) -> Element {
    let (client_state, _) = consume_context::<ClientState>(hooks).unwrap();
    let runtime = hooks.world.resource(runtime()).clone();
    let (instances, set_instances) = use_state(hooks, HashMap::new());
    use_interval(hooks, 1., move || {
        let client_state = client_state.clone();
        let set_instances = set_instances.clone();
        runtime.spawn(async move {
            if let Ok(instances) = client_state.rpc(rpc_get_instances_info, ()).await {
                set_instances(instances.instances);
            }
        });
    });
    FlowRow(
        instances
            .into_iter()
            .sorted_by_key(|x| x.0.clone())
            .map(|(key, instance)| {
                Text::el(format!("\u{f6e6} {} ({} players)", key, instance.n_players))
            })
            .collect(),
    )
    .el()
    .keyboard()
}

#[element_component]
fn TerrainMaterialEditor(hooks: &mut Hooks) -> Element {
    let (value, set_value) = use_remote_persisted_resource(hooks, terrain_material_def()).unwrap();
    let value = value.unwrap_or_default();
    let set_value = cb(move |value| set_value(Some(value)));
    FlowColumn::el([
        EditorPlayerInputHandler.el(),
        ScrollArea::el(
            ScrollAreaSizing::FitChildrenWidth,
            FlowColumn::el([
                FlowRow::el([
                    CopyPasteButtons {
                        value,
                        on_change: set_value.clone(),
                    }
                    .el()
                    .with(margin(), Borders::bottom(STREET).into()),
                    // SelectAndDownloadJsonAssetButton2::<TerrainMaterialDef> {
                    //     asset_type: AssetType::TerrainMaterial,
                    //     on_select_file: Cb::new({
                    //         let set_value = set_value.clone();
                    //         move |value| {
                    //             set_value((**value.random().unwrap()).clone());
                    //         }
                    //     }),
                    // }
                    // .el(),
                ])
                .keyboard(),
                // TerrainMaterialDef::editor(value, set_value, Default::default()),
            ])
            .floating_panel(),
        ),
    ])
    .with(margin(), Borders::even(STREET).into())
}

#[element_component]
fn EditorAtmosphereMode(_hooks: &mut Hooks) -> Element {
    // let (config, set_config) = use_remote_first_component(hooks, world, ArchetypeFilter::new().incl(sun::config()), sun::config());
    // let (latitude, set_latitude) = use_remote_first_component(hooks, world, ArchetypeFilter::new().incl(sun::latitude()), sun::latitude());
    // let (axial_tilt, set_axial_tilt) =
    //     use_remote_first_component(hooks, world, ArchetypeFilter::new().incl(sun::axial_tilt()), sun::axial_tilt());
    // let (hour_of_day, set_hour_of_day) =
    //     use_remote_first_component(hooks, world, ArchetypeFilter::new().incl(sun::hour_of_day()), sun::hour_of_day());
    // let (day_of_year, set_day_of_year) =
    //     use_remote_first_component(hooks, world, ArchetypeFilter::new().incl(sun::day_of_year()), sun::day_of_year());

    // let (config, latitude, axial_tilt, hour_of_day, day_of_year) = match (config, latitude, axial_tilt, hour_of_day, day_of_year) {
    //     (Some(c), Some(l), Some(at), Some(hod), Some(doy)) => (c, l, at, hod, doy),
    //     _ => return Element::new(),
    // };

    // let set_config = Arc::new(move |value| set_config(Some(value)));
    // FlowColumn::el([
    //     EditorPlayerInputHandler.el(),
    //     ScrollArea::el(
    //         FlowColumn::el([
    //             FlowRow::el([
    //                 CopyPasteButtons { value: config.clone(), on_change: Cb(set_config.clone()) }
    //                     .el()
    //                     .set(margin(), Borders::bottom(STREET)),
    //                 // SelectAndDownloadJsonAssetButton2::<sun::Config> {
    //                 //     asset_type: AssetType::Atmosphere,
    //                 //     on_select_file: Cb::new({
    //                 //         let set_config = set_config.clone();
    //                 //         move |config| {
    //                 //             set_config((**config.random().unwrap()).clone());
    //                 //         }
    //                 //     }),
    //                 // }
    //                 // .el(),
    //             ])
    //             .keyboard(),
    //             sun::components_editor(
    //                 (latitude, set_latitude),
    //                 (axial_tilt, set_axial_tilt),
    //                 (hour_of_day, set_hour_of_day),
    //                 (day_of_year, set_day_of_year),
    //             ),
    //             sun::Config::editor(config, Some(Cb(set_config.clone())), Default::default()),
    //         ])
    //         .floating_panel(),
    //     )
    //     .set(margin(), Borders::even(STREET)),
    // ])
    Element::new()
}

#[element_component]
fn NaturalLayersEditor(hooks: &mut Hooks) -> Element {
    let (value, set_value) = use_remote_persisted_resource(hooks, natural_layers()).unwrap();
    let value = value.unwrap_or_else(|| get_default_natural_layers(NaturalsPreset::Mountains));
    let set_value = cb(move |value| set_value(Some(value)));
    FlowColumn::el([
        EditorPlayerInputHandler.el(),
        ScrollArea::el(
            ScrollAreaSizing::FitChildrenWidth,
            FlowColumn::el([
                FlowRow::el([
                    CopyPasteButtons {
                        value,
                        on_change: set_value.clone(),
                    }
                    .el()
                    .with(margin(), Borders::bottom(STREET).into()),
                    // SelectAndDownloadJsonAssetButton2::<Vec<NaturalLayer>> {
                    //     asset_type: AssetType::Biomes,
                    //     on_select_file: Cb::new({
                    //         let set_value = set_value.clone();
                    //         move |value| {
                    //             set_value((**value.random().unwrap()).clone());
                    //         }
                    //     }),
                    // }
                    // .el(),
                ])
                .keyboard(),
                // Vec::<NaturalLayer>::editor(value, set_value, Default::default()),
            ])
            .floating_panel(),
        )
        .with(margin(), Borders::even(STREET).into()),
    ])
}

#[element_component]
fn EditorExperienceMode(_hooks: &mut Hooks) -> Element {
    Element::new()
    // Dock(vec![PlayerKeyboardInputHandler.el(), PlayerHighjackMouse.el(), PlayInnerUI.el()]).el()
}

#[element_component]
pub fn UploadingThumbnailDialog(_: &mut Hooks) -> Element {
    WindowSized(vec![
        Text::el("Uploading thumbnail...").with(translation(), vec3(300., 300., -0.6))
    ])
    .el()
}

#[element_component]
pub fn EditorPlayerInputHandler(_hooks: &mut Hooks) -> Element {
    // let (show_menu, _) = hooks.consume_context::<ShowMenu>().unwrap();
    // if show_menu.0 {
    //     return Element::new();
    // }

    // let (_, flag_as_updated) = use_state(hooks,());
    // let mouse_hijacked =
    //     hooks.consume_context::<PlayerInputChanges>().unwrap().0.query(|pi| {
    //         [pi.editor_camera_rotate, pi.move_left, pi.move_right, pi.move_forward, pi.move_back, pi.jump].into_iter().any(|b| b)
    //     });
    // Group(vec![
    //     EditorPlayerMovementHandler { flag_as_updated: Cb(flag_as_updated) }.el(),
    //     if mouse_hijacked { PlayerHighjackMouse.el() } else { Element::new() },
    // ])
    // .el()
    Element::new()
}

#[element_component]
pub fn EditorPlayerMovementHandler(
    _hooks: &mut Hooks,
    _flag_as_updated: Cb<dyn Fn(()) + Sync + Send>,
) -> Element {
    // let (player_input, _) = hooks.consume_context::<PlayerInputChanges>().unwrap();

    // Element::new()
    //     .listener(
    //         on_app_keyboard_input(),
    //         Arc::new({
    //             let player_input = player_input.clone();
    //             let flag_as_updated = flag_as_updated.clone();

    //             move |_world, _, event| {
    //                 let changed = player_input.change(|pi| {
    //                     if let KeyboardEvent { keycode: Some(key), state, .. } = event {
    //                         let pressed = state == &ElementState::Pressed;
    //                         pi.handle_keyboard_event(*key, pressed)
    //                     } else {
    //                         false
    //                     }
    //                 });
    //                 if changed {
    //                     flag_as_updated(());
    //                 }
    //                 changed
    //             }
    //         }),
    //     )
    //     .listener(
    //         on_app_modifiers_change(),
    //         Arc::new({
    //             let player_input = player_input.clone();
    //             let flag_as_updated = flag_as_updated.clone();

    //             move |_world, _, event| {
    //                 let changed = player_input.change(|pi| {
    //                     let new_sprint = event.shift();
    //                     if pi.sprint != new_sprint {
    //                         pi.sprint = new_sprint;
    //                         true
    //                     } else {
    //                         false
    //                     }
    //                 });
    //                 if changed {
    //                     flag_as_updated(());
    //                 }
    //             }
    //         }),
    //     )
    //     .listener(
    //         on_app_mouse_input(),
    //         Arc::new({
    //             let player_input = player_input.clone();
    //             let flag_as_updated = flag_as_updated.clone();
    //             move |_world, _, event| {
    //                 let MouseInput { state, button } = event;
    //                 if button == &MouseButton::Right {
    //                     player_input.always_change(|pi| pi.editor_camera_rotate = *state == ElementState::Pressed);
    //                     flag_as_updated(());
    //                 }
    //             }
    //         }),
    //     )
    //     .listener(
    //         on_app_mouse_wheel(),
    //         Arc::new({
    //             move |_world, _, delta| {
    //                 process_scroll_wheel_delta(player_input.clone(), delta);
    //                 flag_as_updated(());
    //                 true
    //             }
    //         }),
    //     )
    Element::new()
}

// #[element_component]
// fn UploadThumbnailButton(hooks: &mut Hooks) -> Element {
//     let (client_state, _) = hooks.consume_context::<GameClient>().unwrap();
//     let (world_instance_config, _) = hooks.consume_context::<Option<WorldInstanceConfig>>().unwrap();
//     let (render_target, _) = consume_context::<GameClientRenderTarget>(hooks,).unwrap();
//     Button::new_async("\u{f030}", move || {
//         let client_state = client_state.clone();
//         let reader = render_target.0.color_buffer.reader();
//         let map_url = world_instance_config.as_ref().unwrap().map_url.clone();
//         async move {
//             let screenshot = reader.read_image().await.unwrap().into_rgb8();
//             let aspect = 720. / 1280.;
//             let expected_height = screenshot.width() as f32 * aspect;
//             let cropped = if (screenshot.height() as f32) < expected_height {
//                 // keep height, crop width
//                 let new_width = screenshot.height() as f32 / aspect;
//                 let diff = screenshot.width() as f32 - new_width;
//                 image::imageops::crop_imm(&screenshot, (diff / 2.) as u32, 0, new_width as u32, screenshot.height())
//             } else {
//                 // keep width, crop height
//                 let diff = screenshot.height() as f32 - expected_height;
//                 image::imageops::crop_imm(&screenshot, 0, (diff / 2.) as u32, screenshot.width(), expected_height as u32)
//             }
//             .to_image();
//             let thumbnail = image::imageops::resize(&cropped, 640, 360, image::imageops::FilterType::CatmullRom);

//             // log_network_result!(client_state.rpc(rpc_upload_thumbnail, (map_url, image_to_png(thumbnail))).await);
//             // original: &image_to_png(screenshot)
//         }
//     })
//     .tooltip("Upload thumbnail")
//     .el()
// }

fn _image_to_png(image: RgbImage) -> Vec<u8> {
    let image = DynamicImage::ImageRgb8(image);
    let mut buff = Cursor::new(Vec::new());
    image.write_to(&mut buff, ImageOutputFormat::Png).unwrap();
    buff.into_inner()
}

#[element_component]
pub fn Crosshair(hooks: &mut Hooks) -> Element {
    let (settings, _) = consume_context::<EditorSettings>(hooks).unwrap();
    if !settings.show_hud {
        return Element::new();
    }
    let window_size = use_window_logical_resolution(hooks).as_vec2();
    Rectangle
        .el()
        .with(width(), 2.)
        .with(height(), 2.)
        .with_background(Color::WHITE.into())
        .with(
            translation(),
            vec3(window_size.x / 2. - 1., window_size.y / 2. - 1., -0.01),
        )
}

#[derive(Debug, Clone)]
pub struct CopyPasteButtons<
    T: Serialize + DeserializeOwned + Send + Sync + std::fmt::Debug + Clone + 'static,
> {
    pub value: T,
    pub on_change: Cb<dyn Fn(T) + Send + Sync>,
}
impl<T: Serialize + DeserializeOwned + Send + Sync + std::fmt::Debug + Clone + 'static>
    ElementComponent for CopyPasteButtons<T>
{
    fn render(self: Box<Self>, _hooks: &mut Hooks) -> Element {
        let Self { value, on_change } = *self;
        FlowRow(vec![
            Button::new("Copy", move |_| {
                ambient_sys::clipboard::set_background(
                    serde_json::to_string_pretty(&value).ok().unwrap(),
                    |res| {
                        if let Err(err) = res {
                            tracing::error!("Failed to write to clipboard {err:?}");
                        }
                    },
                );
            })
            .el(),
            Button::new("Paste", move |_| {
                let on_change = on_change.clone();

                ambient_sys::clipboard::get_background(move |res| {
                    if let Some(res) = res {
                        on_change(serde_json::from_str(&res).unwrap());
                    }
                });
            })
            .el(),
        ])
        .el()
        .with(space_between_items(), STREET)
    }
}
