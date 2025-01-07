use std::collections::HashMap;

use ambient_ecs::ArchetypeFilter;
use ambient_native_std::asset_cache::{AssetCache, SyncAssetKey};
use derive_more::Display;
use glam::{Mat4, Vec4};
use itertools::Itertools;
use parking_lot::Mutex;
use wgpu::BindGroupLayoutEntry;

use super::ENTITIES_BIND_GROUP;

pub type GpuComponentId = &'static str;

#[derive(Clone, Debug)]
pub struct GpuComponent {
    pub name: String,
    pub format: GpuComponentFormat,
    pub filter: ArchetypeFilter,
}

/// Represents the type of the component in the shader.
///
/// Each separate type will have its own buffer, but the buffer is shared by all components of the same type.
///
/// **Note**: Prefer to use the existing types and casting, for example, using a Mat4 as an array,
/// rather than adding a new type.
///
/// This is because there is an upper limit (8) to the number of storage buffers for each shader stage.
#[derive(Display, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GpuComponentFormat {
    Mat4,
    Vec4,
}
impl GpuComponentFormat {
    pub fn size(&self) -> u64 {
        match self {
            GpuComponentFormat::Mat4 => std::mem::size_of::<Mat4>() as u64,
            GpuComponentFormat::Vec4 => std::mem::size_of::<Vec4>() as u64,
        }
    }
    pub fn wgsl(&self) -> &'static str {
        match self {
            GpuComponentFormat::Mat4 => "mat4x4<f32>",
            GpuComponentFormat::Vec4 => "vec4<f32>",
        }
    }
}

/// Generates the wgsl for buffers storing the component type
#[derive(Clone, Debug)]
pub struct GpuComponentsConfig {
    /// The primitive type of the component
    pub format: GpuComponentFormat,
    /// The set of components which share the same type which will coexist in the same buffer, using `MultiBuffer`
    pub components: Vec<GpuComponent>,
    pub components_before_this: usize,
}

impl GpuComponentsConfig {
    pub fn new(format: GpuComponentFormat) -> Self {
        Self {
            format,
            components: Vec::new(),
            components_before_this: 0,
        }
    }
    pub fn layout_offset(&self, archetypes: usize) -> usize {
        1 + self.components_before_this * archetypes
    }

    /// Generate bindings for accessing this gpu component type in the specified `bind_group` and `buffer_index`
    fn wgsl(&self, bind_group: &str, buffer_index: u32, writeable: bool) -> String {
        format!(
            "
struct Entity{format_name}Buffer {{ data: array<{wgsl_format}> }};

@group({bind_group})
@binding({data_binding})
var<storage{storage_attr}> entity_{format_name}_data: Entity{format_name}Buffer;

fn get_entity_component_offset_{format_name}(component_index: u32, entity_loc: vec2<u32>) -> i32 {{
    let archetypes = u32(entity_layout.data[0]);
    let layout_offset = 1u + ({components_before_this}u + component_index) * archetypes;
    return entity_layout.data[layout_offset + entity_loc.x];
}}

fn get_entity_data_{format_name}(component_index: u32, entity_loc: vec2<u32>) -> {wgsl_format} {{
    return entity_{format_name}_data.data[u32(get_entity_component_offset_{format_name}(component_index, entity_loc)) + entity_loc.y];
}}
fn get_entity_data_or_{format_name}(component_index: u32, entity_loc: vec2<u32>, default_value: {wgsl_format}) -> {wgsl_format} {{
    let loc = get_entity_component_offset_{format_name}(component_index, entity_loc);
    if (loc >= 0) {{
        return entity_{format_name}_data.data[u32(loc) + entity_loc.y];
    }} else {{
        return default_value;
    }}
}}

{set_entity_data}

{component_getters}
",
            bind_group = bind_group,
            format_name = self.format,
            data_binding = buffer_index + 1,
            wgsl_format = self.format.wgsl(),
            components_before_this = self.components_before_this,
            storage_attr = if writeable { ", read_write" } else { "" },
            set_entity_data = if writeable {
                format!(
                    "

fn set_entity_data_{format_name}(component_index: u32, entity_loc: vec2<u32>, value: {wgsl_format}) {{
    entity_{format_name}_data.data[u32(get_entity_component_offset_{format_name}(component_index, entity_loc)) + entity_loc.y] = value;
}}
",
                    format_name = self.format.to_string(),
                    wgsl_format = self.format.wgsl()
                )
            } else {
                String::new()
            },
            component_getters = self
                .components
                .iter()
                .enumerate()
                .map(|(i, comp)| {
                    let offset = i;
                    let ident = &comp.name;
                    let format = comp.format;
                    let ty = comp.format.wgsl();

                    let getters = format!(
                        "
fn get_entity_{ident}(entity_loc: vec2<u32>) -> {ty} {{
    return get_entity_data_{format}({offset}u, entity_loc);
}}

fn get_entity_{ident}_or(entity_loc: vec2<u32>, default_value: {ty}) -> {ty} {{
    return get_entity_data_or_{format}({offset}u, entity_loc, default_value);
}}

fn has_entity_{ident}(entity_loc: vec2<u32>) -> bool {{
    return get_entity_component_offset_{format}({offset}u, entity_loc) >= 0;
}}
"
                    );
                    let setters = if writeable {
                        format!(
                            "
fn set_entity_{ident}(entity_loc: vec2<u32>, value: {ty}) {{
set_entity_data_{format}({offset}u, entity_loc, value);
}}
"
                        )
                    } else {
                        String::new()
                    };

                    [getters, setters].join("\n")
                })
                .join("")
        )
    }
}

/// Generates the wgsl code for the gpu ecs storage bindings and access functions.
#[derive(Clone, Debug)]
pub struct GpuWorldConfig {
    pub buffers: Vec<GpuComponentsConfig>,
}

impl GpuWorldConfig {
    pub fn new(mut buffers: Vec<GpuComponentsConfig>) -> Self {
        let mut comps = 0;
        for buf in buffers.iter_mut() {
            buf.components_before_this = comps;
            comps += buf.components.len();
        }
        Self { buffers }
    }
    pub fn wgsl(&self, writeable: bool) -> String {
        let buffers = self
            .buffers
            .iter()
            .enumerate()
            .map(|(i, buf)| buf.wgsl(ENTITIES_BIND_GROUP, i as u32, writeable))
            .join("\n");

        format!(
            "
struct EntityLayoutBuffer {{ data: array<i32>, }};
@group({ENTITIES_BIND_GROUP})
@binding(0)
var<storage> entity_layout: EntityLayoutBuffer;

{buffers}
",
        )
    }

    pub fn layout_entries(&self, read_only: bool) -> impl Iterator<Item = BindGroupLayoutEntry> {
        fn entity_component_storage_entry(binding: u32, writeable: bool) -> BindGroupLayoutEntry {
            wgpu::BindGroupLayoutEntry {
                binding,
                visibility: if writeable {
                    wgpu::ShaderStages::COMPUTE
                } else {
                    wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE
                },
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        read_only: !(writeable && binding != 0),
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        }

        (0..self.buffers.len() + 1)
            .map(move |i| entity_component_storage_entry(i as u32, !read_only))
    }
}

#[derive(Debug)]
pub struct GpuWorldConfigKey;
impl SyncAssetKey<GpuWorldConfig> for GpuWorldConfigKey {
    fn load(&self, _assets: AssetCache) -> GpuWorldConfig {
        let registry = GPU_COMPONENT_REGISTRY.lock();
        let mut by_format = HashMap::new();
        let components = &registry.components;
        for comp in components {
            let entry = by_format
                .entry(comp.format)
                .or_insert_with(|| GpuComponentsConfig::new(comp.format));
            entry.components.push(comp.clone());
        }
        GpuWorldConfig::new(by_format.into_values().collect())
    }
}

lazy_static! {
    pub static ref GPU_COMPONENT_REGISTRY: Mutex<GpuComponentRegistry> =
        Mutex::new(GpuComponentRegistry::new());
}

pub struct GpuComponentRegistry {
    pub components: Vec<GpuComponent>,
}
impl GpuComponentRegistry {
    fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }
}

#[macro_export]
macro_rules! gpu_components {
    ( $( $($cpu_component:expr ),+ => $name:ident : $format:expr, )+ ) => {

        mod gpu_components {
            $(
                $crate::paste::paste! {
                    pub fn $name () -> $crate::GpuComponentId {
                        stringify!($name)
                    }
                }
            )*
        }

        static INIT_GPU_COMPONENTS: std::sync::Once = std::sync::Once::new();
        pub fn init_gpu_components() {
            INIT_GPU_COMPONENTS.call_once(|| {
                let mut registry = $crate::GPU_COMPONENT_REGISTRY.lock();
                $(
                    registry.components.push($crate::GpuComponent {
                        name: stringify!($name).to_string(),
                        format: $format,
                        filter: ambient_ecs::ArchetypeFilter::new()
                            $( .incl($cpu_component) )*
                    });
                )*
            });
        }
    };
}
