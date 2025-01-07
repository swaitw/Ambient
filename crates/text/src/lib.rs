use core::fmt;
use std::{ops::Deref, str::FromStr, sync::Arc};

use ambient_core::{
    asset_cache, async_ecs::async_run, gpu, mesh, runtime, transform::*,
    window::window_scale_factor,
};
use ambient_ecs::{
    components, ensure_has_component, generated::text::types::FontStyle, query, Debuggable, Entity,
    SystemGroup,
};
use ambient_gpu::{mesh_buffer::GpuMesh, texture::Texture};
use ambient_layout::{height, max_height, max_width, min_height, min_width, width};
use ambient_native_std::{
    asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt},
    asset_url::AbsAssetUrl,
    cb,
    download_asset::{AssetResult, BytesFromUrl},
    mesh::*,
    unwrap_log_warn,
};
use ambient_renderer::{
    gpu_primitives_lod, gpu_primitives_mesh, material, primitives, renderer_shader, SharedMaterial,
};
use anyhow::Context;
use async_trait::async_trait;
use glam::*;
use glyph_brush::{
    ab_glyph::{Font, FontArc, PxScale, Rect},
    BrushAction, BrushError, GlyphBrush, GlyphBrushBuilder, GlyphCruncher, Section,
};
use parking_lot::Mutex;

use crate::text_material::{get_text_shader, TextMaterial};

mod text_material;

pub use ambient_ecs::generated::text::components::{font_family, font_size, font_style, text};

components!("text", {
    @[Debuggable]
    text_case: TextCase,
    font_arc: Arc<FontArc>,

    glyph_brush: Arc<Mutex<GlyphBrush<GlyphVertex>>>,
    text_texture: Arc<Texture>,
});

#[derive(Debug, Clone, Copy)]
pub enum TextCase {
    AsTyped,
    Uppercase,
    Lowercase,
}
impl Default for TextCase {
    fn default() -> Self {
        Self::AsTyped
    }
}
impl TextCase {
    pub fn format(&self, text: impl Into<String>) -> String {
        let text: String = text.into();
        match self {
            TextCase::AsTyped => text,
            TextCase::Uppercase => text.to_uppercase(),
            TextCase::Lowercase => text.to_lowercase(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FontFamily {
    Default,
    Custom(AbsAssetUrl),
    FontAwesome { solid: bool },
    SourceSansPro,
}

impl fmt::Display for FontFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontFamily::Default => write!(f, "Default"),
            FontFamily::Custom(url) => write!(f, "{url}"),
            FontFamily::FontAwesome { solid: false } => write!(f, "FontAwesome"),
            FontFamily::FontAwesome { solid: true } => write!(f, "FontAwesomeSolid"),
            FontFamily::SourceSansPro => write!(f, "Code"),
        }
    }
}

impl FromStr for FontFamily {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Default" => Ok(Self::Default),
            "FontAwesome" => Ok(Self::FontAwesome { solid: false }),
            "FontAwesomeSolid" => Ok(Self::FontAwesome { solid: true }),
            "Code" => Ok(Self::SourceSansPro),
            url => Ok(Self::Custom(AbsAssetUrl::from_str(url)?)),
        }
    }
}

impl Default for FontFamily {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Clone)]
struct FontDef(FontFamily, FontStyle);

#[async_trait]
impl AsyncAssetKey<Arc<FontArc>> for FontDef {
    async fn load(self, assets: AssetCache) -> Arc<FontArc> {
        match self.0 {
            FontFamily::Default => {
                let font: &'static [u8] = match self.1 {
                    FontStyle::Bold => {
                        include_bytes!("../fonts/Ubuntu/Ubuntu Bold Nerd Font Complete.ttf")
                    }
                    FontStyle::BoldItalic => {
                        include_bytes!("../fonts/Ubuntu/Ubuntu Bold Italic Nerd Font Complete.ttf")
                    }
                    FontStyle::Italic => {
                        include_bytes!("../fonts/Ubuntu/Ubuntu Italic Nerd Font Complete.ttf")
                    }
                    FontStyle::Light => {
                        include_bytes!("../fonts/Ubuntu/Ubuntu Light Nerd Font Complete.ttf")
                    }
                    FontStyle::LightItalic => {
                        include_bytes!("../fonts/Ubuntu/Ubuntu Light Italic Nerd Font Complete.ttf")
                    }
                    FontStyle::Medium => {
                        include_bytes!("../fonts/Ubuntu/Ubuntu Medium Nerd Font Complete.ttf")
                    }
                    FontStyle::MediumItalic => include_bytes!(
                        "../fonts/Ubuntu/Ubuntu Medium Italic Nerd Font Complete.ttf"
                    ),
                    FontStyle::Regular => {
                        include_bytes!("../fonts/Ubuntu/Ubuntu Nerd Font Complete.ttf")
                    }
                };
                Arc::new(FontArc::try_from_slice(font).unwrap())
            }
            FontFamily::FontAwesome { solid } => Arc::new(
                FontArc::try_from_slice(if solid {
                    include_bytes!("../fonts/FontAwesome/Font Awesome 6 Free-Solid-900.otf")
                } else {
                    include_bytes!("../fonts/FontAwesome/Font Awesome 6 Free-Regular-400.otf")
                })
                .unwrap(),
            ),
            FontFamily::SourceSansPro => {
                let font: &'static [u8] = match self.1 {
                    FontStyle::Bold => {
                        include_bytes!("../fonts/Source_Sans_Pro/SourceSansPro-Bold.ttf")
                    }
                    FontStyle::BoldItalic => {
                        include_bytes!("../fonts/Source_Sans_Pro/SourceSansPro-BoldItalic.ttf")
                    }
                    FontStyle::Italic => {
                        include_bytes!("../fonts/Source_Sans_Pro/SourceSansPro-Italic.ttf")
                    }
                    FontStyle::Light => {
                        include_bytes!("../fonts/Source_Sans_Pro/SourceSansPro-Light.ttf")
                    }
                    FontStyle::LightItalic => {
                        include_bytes!("../fonts/Source_Sans_Pro/SourceSansPro-LightItalic.ttf")
                    }
                    FontStyle::Medium => {
                        include_bytes!("../fonts/Source_Sans_Pro/SourceSansPro-SemiBold.ttf")
                    }
                    FontStyle::MediumItalic => {
                        include_bytes!("../fonts/Source_Sans_Pro/SourceSansPro-SemiBoldItalic.ttf")
                    }
                    FontStyle::Regular => {
                        include_bytes!("../fonts/Source_Sans_Pro/SourceSansPro-Regular.ttf")
                    }
                };
                Arc::new(FontArc::try_from_slice(font).unwrap())
            }
            FontFamily::Custom(url) => match FontFromUrl(url.clone()).get(&assets).await {
                Ok(font) => font,
                Err(err) => {
                    tracing::error!("Failed to fetch font at {url}: {err}; using fallback font");
                    FontDef(FontFamily::Default, self.1).get(&assets).await
                }
            },
        }
    }
}

pub fn systems(use_gpu: bool) -> SystemGroup {
    SystemGroup::new(
        "ui/text",
        vec![
            ensure_has_component(text(), font_family(), FontFamily::Default.to_string()),
            ensure_has_component(text(), font_style(), FontStyle::Regular),
            ensure_has_component(text(), font_size(), 12.),
            query(())
                .incl(text())
                .excl(renderer_shader())
                .spawned()
                .to_system(move |q, world, qs, _| {
                    if !use_gpu {
                        return;
                    }

                    let assets = world.resource(asset_cache()).clone();
                    let gpu = world.resource(gpu()).clone();

                    for (id, _) in q.collect_cloned(world, qs) {
                        let texture = Arc::new(Texture::new(
                            &gpu,
                            &wgpu::TextureDescriptor {
                                size: wgpu::Extent3d {
                                    width: 256,
                                    height: 256,
                                    depth_or_array_layers: 1,
                                },
                                mip_level_count: 1,
                                sample_count: 1,
                                dimension: wgpu::TextureDimension::D2,
                                format: wgpu::TextureFormat::R8Unorm,
                                usage: wgpu::TextureUsages::TEXTURE_BINDING
                                    | wgpu::TextureUsages::COPY_DST,
                                label: Some("Text.texture"),
                                view_formats: &[],
                            },
                        ));
                        let texture_view =
                            Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));
                        world
                            .add_components(
                                id,
                                Entity::new()
                                    .with(text_texture(), texture)
                                    .with(renderer_shader(), cb(get_text_shader))
                                    .with(
                                        material(),
                                        SharedMaterial::new(TextMaterial::new(
                                            &gpu,
                                            &assets,
                                            texture_view,
                                        )),
                                    )
                                    .with(primitives(), vec![])
                                    .with(gpu_primitives_mesh(), Default::default())
                                    .with(gpu_primitives_lod(), Default::default()),
                            )
                            .unwrap();
                    }
                }),
            query((font_family().changed(), font_style().changed())).to_system(
                |q, world, qs, _| {
                    for (id, (font_family, font_style)) in q.collect_cloned(world, qs) {
                        let async_run = world.resource(async_run()).clone();
                        let assets = world.resource(asset_cache()).clone();
                        world.resource(runtime()).spawn(async move {
                            let font = FontDef(
                                unwrap_log_warn!(FontFamily::from_str(&font_family)),
                                font_style,
                            )
                            .get(&assets)
                            .await;
                            async_run.run(move |world| {
                                world.add_component(id, font_arc(), font).ok();
                            });
                        });
                    }
                },
            ),
            query(font_arc().changed()).to_system(|q, world, qs, _| {
                for (id, font) in q.collect_cloned(world, qs) {
                    let brush = Arc::new(Mutex::new(
                        GlyphBrushBuilder::using_font(font.deref().clone()).build(),
                    ));
                    world.add_component(id, glyph_brush(), brush).unwrap();
                }
            }),
            query(())
                .incl(mesh_to_local())
                .incl(text())
                .to_system(|q, world, qs, _| {
                    let scale_factor = world
                        .resource_opt(window_scale_factor())
                        .cloned()
                        .unwrap_or(1.) as f32;
                    for (id, _) in q.collect_cloned(world, qs) {
                        world
                            .set_if_changed(
                                id,
                                mesh_to_local(),
                                Mat4::from_scale(Vec3::ONE / scale_factor),
                            )
                            .unwrap();
                    }
                }),
            {
                let q = query((
                    glyph_brush().changed(),
                    text().changed(),
                    font_size().changed(),
                    font_arc(),
                ));
                if use_gpu {
                    q.incl(text_texture())
                } else {
                    q
                }
            }
            .optional_changed(text_case())
            .optional_changed(min_width())
            .to_system(move |q, world, qs, _| {
                let scale_factor = world
                    .resource_opt(window_scale_factor())
                    .cloned()
                    .unwrap_or(1.) as f32;
                for (id, (glyph_brush, text, font_size, font)) in q.collect_cloned(world, qs) {
                    let assets = world.resource(asset_cache()).clone();
                    let text = world.get(id, text_case()).unwrap_or_default().format(text);
                    let min_width = world.get(id, min_width()).unwrap_or(0.);
                    let min_height = world.get(id, min_height()).unwrap_or(0.);
                    let max_width = world.get(id, max_width()).unwrap_or(f32::MAX);
                    let max_height = world.get(id, max_height()).unwrap_or(f32::MAX);

                    loop {
                        let process_result = {
                            let mut brush = glyph_brush.lock();
                            let section = Section::default()
                                .with_bounds((max_width, max_height))
                                .add_text(glyph_brush::Text::new(&text).with_scale(
                                    pt_size_to_px_scale(&*font, font_size, scale_factor),
                                ));
                            if let Some(bounds) = brush.glyph_bounds(&section) {
                                if world.has_component(id, width()) {
                                    world
                                        .set_if_changed(
                                            id,
                                            width(),
                                            (bounds.max.x / scale_factor).max(min_width),
                                        )
                                        .unwrap();
                                }
                                if world.has_component(id, height()) {
                                    world
                                        .set_if_changed(
                                            id,
                                            height(),
                                            (bounds.max.y / scale_factor).max(min_height),
                                        )
                                        .unwrap();
                                }
                            }
                            brush.queue(section);
                            brush.process_queued(
                                |rect, tex_data| {
                                    if !use_gpu {
                                        return;
                                    }
                                    let gpu = world.resource(gpu());

                                    gpu.queue.write_texture(
                                        wgpu::ImageCopyTexture {
                                            texture: &world
                                                .get_ref(id, text_texture())
                                                .unwrap()
                                                .handle,
                                            mip_level: 0,
                                            origin: wgpu::Origin3d {
                                                x: rect.min[0],
                                                y: rect.min[1],
                                                z: 0,
                                            },
                                            aspect: wgpu::TextureAspect::All,
                                        },
                                        tex_data,
                                        wgpu::ImageDataLayout {
                                            offset: 0,
                                            bytes_per_row: Some(rect.width()),
                                            rows_per_image: Some(rect.height()),
                                        },
                                        wgpu::Extent3d {
                                            width: rect.width(),
                                            height: rect.height(),
                                            depth_or_array_layers: 1,
                                        },
                                    );
                                },
                                |vertex_data| GlyphVertex {
                                    tex_coords: vertex_data.tex_coords,
                                    pixel_coords: vertex_data.pixel_coords,
                                },
                            )
                        };
                        match process_result {
                            Ok(BrushAction::Draw(vertices)) => {
                                if vertices.is_empty() {
                                    // Mesh has no vertices. We have to clear any left over GPU state.
                                    world
                                        .remove_components(
                                            id,
                                            vec![primitives().desc(), mesh().desc()],
                                        )
                                        .unwrap();
                                    world.add_component(id, primitives(), vec![]).unwrap();
                                }
                                let mut data = Entity::new();
                                if use_gpu && !vertices.is_empty() {
                                    let cpu_mesh = mesh_from_glyph_vertices(vertices);
                                    let gpu = world.resource(gpu());
                                    data.set(mesh(), GpuMesh::from_mesh(gpu, &assets, &cpu_mesh));
                                }
                                world.add_components(id, data).unwrap();
                                break;
                            }
                            Ok(BrushAction::ReDraw) => {
                                break;
                            }
                            Err(BrushError::TextureTooSmall { suggested }) => {
                                if !use_gpu {
                                    return;
                                }
                                let size = wgpu::Extent3d {
                                    width: suggested.0,
                                    height: suggested.1,
                                    depth_or_array_layers: 1,
                                };
                                let gpu = world.resource(gpu()).clone();
                                let texture = Arc::new(Texture::new(
                                    &gpu,
                                    &wgpu::TextureDescriptor {
                                        size,
                                        mip_level_count: 1,
                                        sample_count: 1,
                                        dimension: wgpu::TextureDimension::D2,
                                        format: wgpu::TextureFormat::R8Unorm,
                                        usage: wgpu::TextureUsages::TEXTURE_BINDING
                                            | wgpu::TextureUsages::COPY_DST,
                                        label: Some("Text.texture"),
                                        view_formats: &[],
                                    },
                                ));
                                glyph_brush.lock().resize_texture(suggested.0, suggested.1);
                                let view = Arc::new(
                                    texture.create_view(&wgpu::TextureViewDescriptor::default()),
                                );
                                world
                                    .add_components(
                                        id,
                                        Entity::new()
                                            .with(
                                                material(),
                                                SharedMaterial::new(TextMaterial::new(
                                                    &gpu,
                                                    &assets,
                                                    view.clone(),
                                                )),
                                            )
                                            .with(text_texture(), texture),
                                    )
                                    .unwrap();
                            }
                        }
                    }
                }
            }),
        ],
    )
}

// From: https://docs.rs/glyph_brush/latest/glyph_brush/ab_glyph/trait.Font.html#units
fn pt_size_to_px_scale<F: Font>(font: &F, pt_size: f32, screen_scale_factor: f32) -> PxScale {
    let px_per_em = pt_size * screen_scale_factor; // * (96.0 / 72.0); // this part is used in the example but seems to make the scale wrong, hence disabled
    let units_per_em = font.units_per_em().unwrap();
    let height = font.height_unscaled();
    PxScale::from(px_per_em * height / units_per_em)
}

#[derive(Clone)]
pub struct GlyphVertex {
    pub tex_coords: Rect,
    pub pixel_coords: Rect,
}

fn mesh_from_glyph_vertices(vertices: Vec<GlyphVertex>) -> Mesh {
    assert!(!vertices.is_empty());
    let mut positions = Vec::new();
    let mut texcoords = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    for vert in vertices.into_iter() {
        let offset = positions.len() as u32;
        positions.push(vec3(vert.pixel_coords.min.x, vert.pixel_coords.min.y, 0.));
        positions.push(vec3(vert.pixel_coords.max.x, vert.pixel_coords.min.y, 0.));
        positions.push(vec3(vert.pixel_coords.min.x, vert.pixel_coords.max.y, 0.));
        positions.push(vec3(vert.pixel_coords.max.x, vert.pixel_coords.max.y, 0.));

        texcoords.push(vec2(vert.tex_coords.min.x, vert.tex_coords.min.y));
        texcoords.push(vec2(vert.tex_coords.max.x, vert.tex_coords.min.y));
        texcoords.push(vec2(vert.tex_coords.min.x, vert.tex_coords.max.y));
        texcoords.push(vec2(vert.tex_coords.max.x, vert.tex_coords.max.y));

        normals.push(vec3(0., 0., 1.));
        normals.push(vec3(0., 0., 1.));
        normals.push(vec3(0., 0., 1.));
        normals.push(vec3(0., 0., 1.));

        indices.push(offset);
        indices.push(offset + 1);
        indices.push(offset + 2);

        indices.push(offset + 1);
        indices.push(offset + 3);
        indices.push(offset + 2);
    }
    MeshBuilder {
        positions,
        texcoords: vec![texcoords],
        normals,
        indices,
        ..MeshBuilder::default()
    }
    .build()
    .expect("Invalid glyph mesh")
}

#[derive(Debug, Clone)]
pub struct FontFromUrl(AbsAssetUrl);

#[async_trait]
impl AsyncAssetKey<AssetResult<Arc<FontArc>>> for FontFromUrl {
    async fn load(
        self,
        assets: ambient_native_std::asset_cache::AssetCache,
    ) -> AssetResult<Arc<FontArc>> {
        tracing::info!("Downloading font: {}", self.0);
        let data = BytesFromUrl::new(self.0, true).get(&assets).await?;
        let brush = FontArc::try_from_vec(data.deref().clone()).context("Failed to parse font")?;
        Ok(Arc::new(brush))
    }
}
