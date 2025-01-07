use std::{borrow::Cow, collections::BTreeMap, ops::Deref, sync::Arc};

use aho_corasick::AhoCorasick;
use ambient_native_std::{asset_cache::*, CowStr};
use ambient_std::topological_sort::{topological_sort, TopologicalSortable};
use anyhow::Context;
use itertools::Itertools;
use wgpu::{
    BindGroupLayout, BindGroupLayoutEntry, ComputePipelineDescriptor, DepthBiasState, TextureFormat,
};

use super::gpu::{Gpu, GpuKey, DEFAULT_SAMPLE_COUNT};

#[derive(Debug, Clone, PartialEq)]
pub enum WgslValue {
    String(CowStr),
    Raw(CowStr),
    Float(f32),
    Int32(u32),
    Int64(u64),
}

impl WgslValue {
    pub fn as_integer(&self) -> Option<u32> {
        match self {
            WgslValue::Int32(v) => Some(*v),
            _ => None,
        }
    }

    fn to_wgsl(&self) -> String {
        match self {
            WgslValue::String(v) => format!("{v:?}"),
            WgslValue::Raw(v) => v.to_string(),
            WgslValue::Float(v) => v.to_string(),
            WgslValue::Int32(v) => v.to_string(),
            WgslValue::Int64(v) => v.to_string(),
        }
    }
}

impl From<&'static str> for WgslValue {
    fn from(v: &'static str) -> Self {
        Self::String(v.into())
    }
}
impl From<String> for WgslValue {
    fn from(v: String) -> Self {
        Self::String(v.into())
    }
}

impl From<f32> for WgslValue {
    fn from(v: f32) -> Self {
        Self::Float(v)
    }
}

impl From<u32> for WgslValue {
    fn from(v: u32) -> Self {
        Self::Int32(v)
    }
}

impl From<u64> for WgslValue {
    fn from(v: u64) -> Self {
        Self::Int64(v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShaderIdent {
    name: CowStr,
    value: WgslValue,
}

impl ShaderIdent {
    /// Shortcut for unescaped text replacement
    pub fn raw(name: impl Into<CowStr>, value: impl Into<CowStr>) -> Self {
        Self {
            name: name.into(),
            value: WgslValue::Raw(value.into()),
        }
    }

    /// Replaces any occurence of `name` with the wgsl representation of `value`
    pub fn constant(name: impl Into<CowStr>, value: impl Into<WgslValue>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

type BindingEntry = (CowStr, BindGroupLayoutEntry);

/// Defines a part of a shader, with preprocessing.
///
/// Each shadermodule contains:
/// - Source code
/// - Dependencies
/// - Identifier, used for preprocessing and replacing, such as constants
/// - A list of binding entries for generating the complete pipeline layout when the shader is assembled.
///     The bindings *do not* describe complete binding groups, as they may be spread out over several shader modules.
///
///     As such, it is not possible to get the bind group layout from a single shader module. Prefer to split out and reuse the entries in a separate function
#[derive(Debug, Default)]
pub struct ShaderModule {
    /// The unique name of the shadermodule.
    pub name: CowStr,
    /// The wgsl source for the module, *without* dependencies
    pub source: CowStr,

    /// Dependencies for the module
    pub dependencies: Vec<Arc<ShaderModule>>,

    // Use the label to preprocess constants
    pub idents: Vec<ShaderIdent>,
    bindings: Vec<BindingEntry>,
}

impl ShaderModule {
    pub fn new(name: impl Into<CowStr>, source: impl Into<CowStr>) -> Self {
        Self {
            name: name.into(),
            source: source.into(),
            idents: Default::default(),
            bindings: Default::default(),
            dependencies: Default::default(),
        }
    }

    pub fn with_ident(mut self, ident: ShaderIdent) -> Self {
        self.idents.push(ident);
        self
    }

    pub fn with_binding(mut self, group: impl Into<CowStr>, entry: BindGroupLayoutEntry) -> Self {
        self.bindings.push((group.into(), entry));
        self
    }

    pub fn with_bindings(
        mut self,
        bindings: impl IntoIterator<Item = (CowStr, BindGroupLayoutEntry)>,
    ) -> Self {
        self.bindings.extend(bindings);
        self
    }

    pub fn with_binding_desc(mut self, desc: BindGroupDesc<'static>) -> Self {
        let group = desc.label.clone();
        self.bindings
            .extend(desc.entries.iter().map(|&entry| (group.clone(), entry)));
        self
    }

    pub fn with_dependency(mut self, module: Arc<ShaderModule>) -> Self {
        self.dependencies.push(module);
        self
    }

    pub fn with_dependencies(
        mut self,
        modules: impl IntoIterator<Item = Arc<ShaderModule>>,
    ) -> Self {
        self.dependencies.extend(modules);
        self
    }

    fn sanitized_label(&self) -> String {
        self.name.replace(
            |v: char| !v.is_ascii_alphanumeric() && !"_-.".contains(v),
            "?",
        )
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BindGroupDesc<'a> {
    pub entries: Vec<wgpu::BindGroupLayoutEntry>,
    // Name for group preprocessor
    pub label: Cow<'a, str>,
}

impl<'a> SyncAssetKey<Arc<wgpu::BindGroupLayout>> for BindGroupDesc<'a> {
    fn load(&self, assets: AssetCache) -> Arc<wgpu::BindGroupLayout> {
        let gpu = GpuKey.get(&assets);

        let layout = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(&*self.label),
                entries: &self.entries,
            });

        Arc::new(layout)
    }
}

/// Returns all shader modules in the dependency graph in topological order
///
/// # Panics
///
/// If the dependency graph contains a cycle
fn resolve_module_graph<'a>(roots: &[&'a ShaderModule]) -> Vec<&'a ShaderModule> {
    impl<'a> TopologicalSortable<()> for &'a ShaderModule {
        fn dependencies(&self, _ctx: &()) -> Vec<Self> {
            self.dependencies.iter().map(|v| v.as_ref()).collect()
        }

        fn id(&self, _ctx: &()) -> String {
            self.name.to_string()
        }
    }

    topological_sort(roots.iter().copied(), &()).unwrap()
}

/// Represents a shader and its layout
pub struct Shader {
    module: wgpu::ShaderModule,
    // Ordered sets
    bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
    label: CowStr,
}

impl std::ops::Deref for Shader {
    type Target = wgpu::ShaderModule;

    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

impl Shader {
    pub fn new(
        assets: &AssetCache,
        label: impl Into<CowStr>,
        bind_group_names: &[&str],
        module: &ShaderModule,
    ) -> anyhow::Result<Arc<Self>> {
        let label = label.into();
        let gpu = GpuKey.get(assets);

        let _span = tracing::debug_span!("Shader::from_modules", ?label).entered();

        // The complete dependency graph, in the correct order
        let modules = resolve_module_graph(&[module]);

        // Resolve all bind groups, resolving the names to an index
        let bind_group_index: BTreeMap<_, _> = bind_group_names
            .iter()
            .enumerate()
            .map(|(a, &b)| (b, a))
            .collect();
        let mut bind_groups = bind_group_names
            .iter()
            .map(|group| BindGroupDesc {
                label: Cow::Borrowed(*group),
                entries: Default::default(),
            })
            .collect_vec();

        for module in &modules {
            for (group, binding) in &module.bindings {
                let index = *bind_group_index.get(&**group).with_context(|| {
                    format!("Failed to resolve bind group {group} in {}", module.name)
                })?;

                let desc = &mut bind_groups[index];
                desc.entries.push(*binding);
            }
        }

        // Now for the fun part: constructing the binding group layout descriptors
        let bind_group_layouts = bind_groups
            .iter()
            .map(|desc| desc.get(assets))
            .collect_vec();
        if bind_group_layouts.len() > 4 {
            anyhow::bail!(
                "Maximum bind group layout count exceeded. Expected a maximum of 4, found {}: {bind_group_names:?}",
                bind_group_layouts.len()
            );
        }

        // Efficiently replace all identifiers
        let (patterns, replace_with): (Vec<_>, Vec<_>) = modules
            .iter()
            .flat_map(|v| {
                v.idents
                    .iter()
                    .map(|ShaderIdent { name, value }| (format!("{name}"), value.to_wgsl()))
            })
            .chain(
                bind_group_index
                    .iter()
                    .map(|(name, &index)| (name.to_string(), (index as u32).to_string())),
            )
            .chain([(
                "ZERO_INTEGER_ON_WEB_FLOAT_ON_NATIVE".to_string(),
                // HACKFIX: https://github.com/AmbientRun/Ambient/issues/1098
                // Remove once https://github.com/gfx-rs/naga/issues/2582 is fixed
                if cfg!(target_os = "unknown") {
                    "0"
                } else {
                    "0.0"
                }
                .to_string(),
            )])
            .unzip();

        tracing::debug!(
            "Preprocessing shader using {}",
            patterns
                .iter()
                .zip_eq(&replace_with)
                .map(|(a, b)| { format!("{a} => {b}") })
                .format("\n")
        );

        // Collect the raw source code
        let source = {
            let source = modules
                .iter()
                .map(|module| {
                    let div = "--------------------------------";
                    let label = module.sanitized_label();
                    let source = &module.source;
                    format!("// {div}\n// @module: {label}\n// {div}\n{source}")
                })
                .join("\n\n");

            AhoCorasick::new(patterns)?.replace_all(&source, &replace_with)
        };

        #[cfg(all(not(target_os = "unknown"), debug_assertions))]
        {
            let path = format!("tmp/{label}.wgsl");
            std::fs::create_dir_all("tmp/").unwrap();
            std::fs::write(path, source.as_bytes()).unwrap();
        }

        let module = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&label),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });

        Ok(Arc::new(Self {
            module,
            bind_group_layouts,
            label,
        }))
    }

    #[inline]
    pub fn layouts(&self) -> &[Arc<BindGroupLayout>] {
        &self.bind_group_layouts
    }

    /// The wgpu shader module
    #[inline]
    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }

    pub fn to_pipeline(
        self: &Arc<Self>,
        gpu: &Gpu,
        info: GraphicsPipelineInfo,
    ) -> GraphicsPipeline {
        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&self.label),
                bind_group_layouts: &self.layouts().iter().map(|v| &**v).collect_vec(),
                push_constant_ranges: &[],
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&self.label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: self.module(),
                    entry_point: info.vs_main,
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    front_face: info.front_face,
                    cull_mode: info.cull_mode,
                    topology: info.topology,
                    ..Default::default()
                },
                fragment: Some(wgpu::FragmentState {
                    module: self.module(),
                    entry_point: info.fs_main,
                    targets: info.targets,
                }),
                depth_stencil: info.depth,
                multisample: wgpu::MultisampleState {
                    count: DEFAULT_SAMPLE_COUNT,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        GraphicsPipeline {
            pipeline,
            name: self.label.deref().into(),
            shader: self.clone(),
        }
    }

    pub fn to_compute_pipeline(self: &Arc<Self>, gpu: &Gpu, entry_point: &str) -> ComputePipeline {
        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&self.label),
                bind_group_layouts: &self.layouts().iter().map(|v| &**v).collect_vec(),
                push_constant_ranges: &[],
            });

        let pipeline = gpu
            .device
            .create_compute_pipeline(&ComputePipelineDescriptor {
                label: Some(&self.label),
                layout: Some(&layout),
                module: self.module(),
                entry_point,
            });

        ComputePipeline {
            pipeline,
            shader: self.clone(),
            name: self.label.deref().into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphicsPipelineInfo<'a> {
    pub vs_main: &'a str,
    pub fs_main: &'a str,
    pub depth: Option<wgpu::DepthStencilState>,
    pub targets: &'a [Option<wgpu::ColorTargetState>],
    pub front_face: wgpu::FrontFace,
    pub cull_mode: Option<wgpu::Face>,
    pub topology: wgpu::PrimitiveTopology,
}

impl<'a> Default for GraphicsPipelineInfo<'a> {
    fn default() -> Self {
        Self {
            vs_main: "vs_main",
            fs_main: "fs_main",
            depth: None,
            targets: &[],
            front_face: wgpu::FrontFace::Cw,
            cull_mode: None,
            topology: wgpu::PrimitiveTopology::TriangleList,
        }
    }
}

pub type GraphicsPipeline = Pipeline<wgpu::RenderPipeline>;
pub type ComputePipeline = Pipeline<wgpu::ComputePipeline>;

pub struct Pipeline<P> {
    pipeline: P,
    shader: Arc<Shader>,
    name: String,
}

impl<P> Pipeline<P> {
    /// Get a reference to the graphics pipeline's pipeline.
    pub fn pipeline(&self) -> &P {
        &self.pipeline
    }

    /// Get a reference to the pipeline's shader.
    #[must_use]
    pub fn shader(&self) -> &Shader {
        self.shader.as_ref()
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl<P> std::ops::Deref for Pipeline<P> {
    type Target = Shader;

    fn deref(&self) -> &Self::Target {
        &self.shader
    }
}

#[cfg(not(target_os = "unknown"))]
pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;
#[cfg(target_os = "unknown")]
// HACK: float depth are broken on wgpu:
// stencilLoadOp is (LoadOp::Load) and stencilStoreOp is (StoreOp::Store) when stencilReadOnly (0) or the attachment ([TextureView "Renderer.shadow_target_views" of Texture "Renderer.shadow_texture"]) has no stencil aspect.
// - While validating depthStencilAttachment.
// - While encoding [CommandEncoder].BeginRenderPass([RenderPassDescriptor "Shadow cascade 0"]).

// Adding a stencil part crashes the gpu
pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

impl<'a> GraphicsPipelineInfo<'a> {
    pub fn with_depth(self) -> GraphicsPipelineInfo<'a> {
        Self {
            depth: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                // This is Greater because we're using reverse-z NDC
                depth_compare: wgpu::CompareFunction::Greater,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            ..self
        }
    }

    pub fn with_depth_bias(mut self, state: DepthBiasState) -> GraphicsPipelineInfo<'a> {
        self.depth
            .as_mut()
            .expect("Attempt to set depth bias without a depth buffer")
            .bias = state;
        self
    }
}
