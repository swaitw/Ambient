use std::sync::Arc;

use ambient_core::{
    asset_cache, gpu, runtime,
    transform::{scale, translation},
    window::get_mouse_clip_space_position,
};
use ambient_decals::DecalShaderKey;
use ambient_ecs::{generated::messages, query, ArchetypeFilter};
use ambient_element::{
    consume_context, use_frame, use_interval, use_runtime_message, use_spawn, use_state, Element,
    ElementComponent, ElementComponentExt, Group, Hooks,
};
use ambient_gpu::{
    gpu::Gpu,
    shader_module::{BindGroupDesc, ShaderModule},
};
use ambient_intent::client_push_intent;
use ambient_native_std::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt},
    cb, friendly_id,
};
use ambient_network::client::ClientState;
use ambient_physics::{
    intersection::{rpc_pick, RaycastFilter},
    ColliderScene,
};
use ambient_primitives::Cube;
use ambient_renderer::{
    color, material, renderer_shader, Material, MaterialShader, SharedMaterial, MATERIAL_BIND_GROUP,
};
use ambient_shared_types::{MouseButton, VirtualKeyCode};
use ambient_terrain::{
    brushes::{
        Brush, BrushShape, BrushSize, BrushSmoothness, BrushStrength, HydraulicErosionConfig,
        TerrainBrushStroke,
    },
    intent_terrain_stroke, terrain_world_cell,
};
use ambient_ui_native::{
    margin, space_between_items, Borders, Button, FlowColumn, FlowRow, FontAwesomeIcon, Separator,
    Slider, StylesExt, Text, UIBase, UIExt, WindowSized, STREET,
};
use glam::{vec3, Vec3, Vec3Swizzles, Vec4};
use wgpu::{util::DeviceExt, BindGroup};

use super::EditorPlayerInputHandler;

#[derive(Clone, Debug)]
pub struct TerrainRaycastPicker {
    pub filter: RaycastFilter,
    pub layer: u32,
    pub brush: Brush,
    pub brush_size: BrushSize,
    pub brush_strength: BrushStrength,
    pub brush_shape: BrushShape,
    pub brush_smoothness: BrushSmoothness,
    pub erosion_config: HydraulicErosionConfig,
}
impl ElementComponent for TerrainRaycastPicker {
    fn render(self: Box<Self>, hooks: &mut ambient_element::Hooks) -> Element {
        let action_button = ambient_shared_types::MouseButton::Left;

        let Self {
            filter,
            layer,
            brush,
            brush_size,
            brush_strength,
            brush_smoothness,
            brush_shape,
            erosion_config,
        } = *self;
        let (client_state, _) = consume_context::<ClientState>(hooks).unwrap();
        let (target_position, set_target_position) = use_state(hooks, None);
        let (mouseover, set_mouseover) = use_state(hooks, false);
        let (mousedown, set_mousedown) = use_state::<Option<Vec3>>(hooks, None); // start position

        let (vis_brush_id, set_vis_brush_id) = use_state(hooks, None);

        let game_state = client_state.game_state.clone();
        use_spawn(hooks, move |ui_world| {
            let assets = ui_world.resource(asset_cache());
            let gpu = ui_world.resource(gpu());
            let new_vis_brush_id = Cube
                .el()
                .with(color(), Vec4::ONE)
                .with(translation(), Vec3::Z)
                .with(scale(), Vec3::ONE)
                .with(
                    renderer_shader(),
                    cb(|assets, config| {
                        DecalShaderKey {
                            material_shader: BrushCursorShaderMaterialKey.get(assets),
                            lit: false,
                            shadow_cascades: config.shadow_cascades,
                        }
                        .get(assets)
                    }),
                )
                .with(
                    material(),
                    SharedMaterial::new(BrushCursorMaterial::new(gpu, assets)),
                )
                .spawn_static(&mut game_state.lock().world);

            set_vis_brush_id(Some(new_vis_brush_id));
            move |_ui_world| {
                game_state.lock().world.despawn(new_vis_brush_id);
            }
        });
        use_runtime_message::<messages::WindowMouseInput>(hooks, {
            let set_mousedown = set_mousedown.clone();
            move |_world, event| {
                if !event.pressed && MouseButton::from(event.button) == action_button {
                    set_mousedown(None);
                }
            }
        });
        use_frame(hooks, move |world| {
            let gpu = world.resource(gpu());
            let mouse_clip_pos = get_mouse_clip_space_position(world);
            let mut state = client_state.game_state.lock();

            // Update the target position with the result of our raycast.
            {
                let ray = state.screen_ray(mouse_clip_pos);
                let filter = filter.clone();
                let set_target_position = set_target_position.clone();
                let client_state = client_state.clone();
                world.resource(runtime()).clone().spawn(async move {
                    if let Ok(resp) = client_state.rpc(rpc_pick, (ray, filter)).await {
                        set_target_position(resp.map(|(_, dist)| ray.origin + ray.dir * dist));
                    }
                });
            }

            // If we have a target position, update our visualisation brush and issue
            // terrain brush strokes if the user's mouse is active.
            if let Some(target_position) = target_position {
                if let Some(vis_brush_id) = vis_brush_id {
                    let brush_size = brush_size.0;
                    state
                        .world
                        .set(vis_brush_id, translation(), target_position)
                        .unwrap();
                    state
                        .world
                        .set(vis_brush_id, scale(), Vec3::ONE * brush_size)
                        .unwrap();
                    if brush_size != 0.0 {
                        if let Ok(material) = state.world.get_ref(vis_brush_id, material()) {
                            let picker_material: &BrushCursorMaterial = material.borrow_downcast();
                            picker_material.upload_params(
                                gpu,
                                brush,
                                target_position,
                                brush_size,
                                brush_strength.0,
                                brush_shape,
                            );
                        }
                    }
                }

                if let Some(start_position) = mousedown {
                    let center = target_position.xy();

                    let erosion = erosion_config.clone();
                    let client_state = client_state.clone();
                    world.resource(runtime()).spawn({
                        client_push_intent(
                            client_state,
                            intent_terrain_stroke(),
                            TerrainBrushStroke {
                                center,
                                layer,
                                brush,
                                brush_size,
                                brush_strength,
                                brush_smoothness,
                                brush_shape,
                                start_position,
                                erosion,
                            },
                            None,
                            None,
                        )
                    });
                }
            }
        });

        UIBase
            .el()
            .with_clickarea()
            .on_mouse_enter(closure!(clone set_mouseover, |_, _| { set_mouseover(true) }))
            .on_mouse_leave(closure!(clone set_mouseover, |_, _| { set_mouseover(false); }))
            .on_mouse_down(closure!(clone set_mousedown, |_, _, button| {
                if mouseover && button == action_button {
                    set_mousedown(Some(target_position.unwrap_or_default()));
                }
            }))
            .el()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BrushCursorMaterialParams {
    pub brush_position: Vec3,
    pub brush: Brush,
    pub brush_radius: f32,
    // remapped to be between 0 to 1
    pub brush_strength: f32,
    pub shape: BrushShape,

    pub _padding: f32,
}
impl Default for BrushCursorMaterialParams {
    fn default() -> Self {
        Self {
            brush_position: Vec3::ZERO,
            brush: Brush::Raise,
            brush_radius: 1.,
            brush_strength: 1.,
            shape: BrushShape::Circle,
            _padding: Default::default(),
        }
    }
}

fn get_brush_cursor_layout() -> BindGroupDesc<'static> {
    BindGroupDesc {
        entries: vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: MATERIAL_BIND_GROUP.into(),
    }
}

#[derive(Debug)]
pub struct BrushCursorShaderMaterialKey;
impl SyncAssetKey<Arc<MaterialShader>> for BrushCursorShaderMaterialKey {
    fn load(&self, _assets: AssetCache) -> Arc<MaterialShader> {
        Arc::new(MaterialShader {
            id: "BrushCursorShaderMaterial".to_string(),
            shader: Arc::new(
                ShaderModule::new("BrushCursor", [include_str!("brush_cursor.wgsl")].concat())
                    .with_binding_desc(get_brush_cursor_layout()),
            ),
        })
    }
}

#[derive(Debug)]
pub struct BrushCursorMaterial {
    buffer: wgpu::Buffer,
    id: String,
    pub bind_group: wgpu::BindGroup,
}
impl BrushCursorMaterial {
    pub fn new(gpu: &Gpu, assets: &AssetCache) -> Self {
        let layout = get_brush_cursor_layout().get(assets);

        let buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("BrushCursorMaterial.buffer"),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::bytes_of(&BrushCursorMaterialParams::default()),
            });
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            }],
            label: Some("BrushCursorMaterial.bind_group"),
        });
        Self {
            buffer,
            id: friendly_id(),
            bind_group,
        }
    }
    pub fn upload_params(
        &self,
        gpu: &Gpu,
        brush: Brush,
        brush_position: Vec3,
        brush_radius: f32,
        brush_strength: f32,
        shape: BrushShape,
    ) {
        // This code assumes that the range is from 0.1 to 10.
        let brush_strength = ((brush_strength.log10() + 1.0) / 2.0).clamp(0.0, 1.0);
        gpu.queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::bytes_of(&BrushCursorMaterialParams {
                brush,
                brush_position,
                brush_radius,
                brush_strength,
                shape,
                ..Default::default()
            }),
        );
    }
}
impl Material for BrushCursorMaterial {
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn transparency_group(&self) -> Option<i32> {
        Some(-90)
    }
}

#[derive(Debug, Clone)]
pub struct EditorTerrainMode;
impl ElementComponent for EditorTerrainMode {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let (brush, set_brush) = consume_context::<Brush>(hooks).unwrap();
        let (layer, set_layer) = consume_context::<u32>(hooks).unwrap();
        let (brush_size, set_brush_size) = consume_context::<BrushSize>(hooks).unwrap();
        let (brush_strength, set_brush_strength) = consume_context::<BrushStrength>(hooks).unwrap();
        let (brush_shape, set_brush_shape) = consume_context::<BrushShape>(hooks).unwrap();
        let (brush_smoothness, set_brush_smoothness) =
            consume_context::<BrushSmoothness>(hooks).unwrap();
        let (erosion_config, _set_erosion_config) =
            consume_context::<HydraulicErosionConfig>(hooks).unwrap();

        let mut items = vec![
            EditorPlayerInputHandler.el(),
            Button::new_value(
                FontAwesomeIcon::el(0xf35b, true),
                brush,
                set_brush.clone(),
                Brush::Raise,
            )
            .hotkey(VirtualKeyCode::Key1)
            .tooltip("Raise")
            .el(),
            Button::new_value(
                FontAwesomeIcon::el(0xf358, true),
                brush,
                set_brush.clone(),
                Brush::Lower,
            )
            .hotkey(VirtualKeyCode::Key2)
            .tooltip("Lower")
            .el(),
            Button::new_value(
                FontAwesomeIcon::el(0xf056, true),
                brush,
                set_brush.clone(),
                Brush::Flatten,
            )
            .hotkey(VirtualKeyCode::Key3)
            .tooltip("Flatten")
            .el(),
            Button::new_value(
                FontAwesomeIcon::el(0xf043, true),
                brush,
                set_brush.clone(),
                Brush::Erode,
            )
            .hotkey(VirtualKeyCode::Key4)
            .tooltip("Hydraulic Erosion")
            .el(),
            Button::new_value(
                FontAwesomeIcon::el(0xf185, true),
                brush,
                set_brush.clone(),
                Brush::Thermal,
            )
            .hotkey(VirtualKeyCode::Key5)
            .tooltip("Thermal Erosion")
            .el(),
            Separator { vertical: true }.el(),
            FlowRow(vec![
                Text::el("Size"),
                Slider {
                    value: brush_size.0,
                    on_change: Some(cb(
                        closure!(clone set_brush_size, |value| set_brush_size(BrushSize(value))),
                    )),
                    min: 1.0,
                    max: 500.0,
                    width: 200.0,
                    logarithmic: true,
                    round: Some(2),
                    suffix: Some(" m"),
                }
                .el(),
            ])
            .el()
            .with(space_between_items(), STREET),
        ];
        if let Brush::Raise | Brush::Lower | Brush::Flatten = brush {
            items.push(
                FlowRow(vec![
                    Text::el("Strength"),
                    Slider {
                        value: brush_strength.0,
                        on_change: Some(cb(closure!(clone set_brush_strength, |value| set_brush_strength(BrushStrength(value))))),
                        min: BrushStrength::SMALL.0,
                        max: BrushStrength::LARGE.0,
                        width: 200.0,
                        logarithmic: true,
                        round: Some(2),
                        suffix: None,
                    }
                    .el(),
                ])
                .el()
                .with(space_between_items(), STREET),
            );
            items.push(
                FlowRow(vec![
                    Text::el("Smoothness"),
                    Slider {
                        value: brush_smoothness.0,
                        on_change: Some(cb(closure!(clone set_brush_smoothness, |value| set_brush_smoothness(BrushSmoothness(value))))),
                        min: 0.,
                        max: 1.,
                        width: 200.0,
                        logarithmic: false,
                        round: Some(2),
                        suffix: None,
                    }
                    .el(),
                ])
                .el()
                .with(space_between_items(), STREET),
            );
            items.push(Separator { vertical: true }.el());
            items.push(
                Button::new_value(
                    FontAwesomeIcon::el(0xf111, true),
                    brush_shape,
                    set_brush_shape.clone(),
                    BrushShape::Circle,
                )
                .hotkey(VirtualKeyCode::Z)
                .tooltip("Circle Shape")
                .el(),
            );
            items.push(
                Button::new_value(
                    FontAwesomeIcon::el(0xf0c8, true),
                    brush_shape,
                    set_brush_shape.clone(),
                    BrushShape::Square,
                )
                .hotkey(VirtualKeyCode::X)
                .tooltip("Square Shape")
                .el(),
            );
            if let Brush::Raise | Brush::Lower = brush {
                items.push(Separator { vertical: true }.el());
                items.push(
                    Button::new_value(
                        FontAwesomeIcon::el(0xf6fc, true),
                        layer,
                        set_layer.clone(),
                        0,
                    )
                    .hotkey(VirtualKeyCode::C)
                    .tooltip("Rock")
                    .el(),
                );
                items.push(
                    Button::new_value(
                        FontAwesomeIcon::el(0xe43b, true),
                        layer,
                        set_layer.clone(),
                        1,
                    )
                    .hotkey(VirtualKeyCode::V)
                    .tooltip("Soil")
                    .el(),
                );
            }
        }

        WindowSized(vec![
            FlowColumn::el([FlowRow(items)
                .el()
                .floating_panel()
                .keyboard()
                .with(margin(), Borders::even(STREET).into())]),
            Group(vec![WindowSized(
                TerrainRaycastPicker {
                    filter: RaycastFilter {
                        entities: Some(ArchetypeFilter::new().incl(terrain_world_cell())),
                        collider_type: Some(ColliderScene::Physics),
                    },
                    layer,
                    brush,
                    brush_size,
                    brush_strength,
                    brush_smoothness,
                    brush_shape,
                    erosion_config,
                }
                .el()
                .vec_of(),
            )
            .el()])
            .el()
            .with(translation(), vec3(0., 0., -10.0)),
        ])
        .el()
    }
}

#[derive(Debug, Clone)]
pub struct GenerateTerrainButton;
impl ElementComponent for GenerateTerrainButton {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let (client_state, _) = consume_context::<ClientState>(hooks).unwrap();
        let (has_terrain, set_has_terrain) = use_state(hooks, true);
        use_interval(hooks, 1., {
            let client_state = client_state.clone();
            move || {
                let state = client_state.game_state.lock();
                let has_terrain = query((terrain_world_cell(),))
                    .iter(&state.world, None)
                    .count()
                    != 0;
                set_has_terrain(has_terrain);
            }
        });
        if !has_terrain {
            Button::new("Generate terrain", move |world| {
                world.resource(runtime()).spawn(client_push_intent(
                    client_state.clone(),
                    intent_terrain_stroke(),
                    TerrainBrushStroke::initial_island(),
                    None,
                    None,
                ));
            })
            .el()
        } else {
            Element::new()
        }
    }
}
