use ambient_native_std::asset_cache::{AssetCache, SyncAssetKeyExt};

use crate::gpu::Gpu;

use super::blit::BlitterKey;

// From: https://github.com/gfx-rs/wgpu/blob/master/wgpu/examples/mipmap/main.rs

pub fn generate_mipmaps(
    gpu: &Gpu,
    assets: &AssetCache,
    encoder: &mut wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    format: wgpu::TextureFormat,
    mip_count: u32,
    layer: u32,
) {
    let blitter = BlitterKey {
        format: format.into(),
        min_filter: wgpu::FilterMode::Linear,
        gamma_correction: None,
    }
    .get(assets);

    let views = (0..mip_count)
        .map(|mip| {
            texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("mip"),
                format: None,
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: mip,
                mip_level_count: Some(1),
                base_array_layer: layer,
                array_layer_count: Some(1),
            })
        })
        .collect::<Vec<_>>();

    for target_mip in 1..mip_count as usize {
        blitter.run(gpu, encoder, &views[target_mip - 1], &views[target_mip]);
    }
}
