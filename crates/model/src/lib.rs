use std::{collections::HashMap, str::FromStr, sync::Arc};

use ambient_core::{
    asset_cache,
    async_ecs::{async_run, AsyncRun},
    bounding::{local_bounding_aabb, world_bounding_aabb, world_bounding_sphere},
    gpu,
    hierarchy::{children, despawn_recursive},
    main_scene, runtime,
    transform::{get_world_position, local_to_world, mesh_to_world},
};
use ambient_ecs::{
    components, query, ComponentDesc, Debuggable, Entity, EntityId, MaybeResource, Networked,
    Store, SystemGroup, World,
};
use ambient_gpu::mesh_buffer::GpuMeshFromUrl;
use ambient_native_std::{
    asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKey, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, AssetUrl, ModelAssetType, TypedAssetUrl},
    cb,
    download_asset::{AssetError, BytesFromUrl},
    log_result,
    math::Line,
};
use ambient_renderer::{
    color, gpu_primitives_lod, gpu_primitives_mesh,
    materials::{
        flat_material::{get_flat_shader, FlatMaterialKey},
        pbr_material::get_pbr_shader,
    },
    pbr_material::PbrMaterialFromUrl,
    primitives, RenderPrimitive, StandardShaderKey,
};
use async_trait::async_trait;
use futures::StreamExt;
use glam::{vec4, Vec3};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
mod model;

use ambient_meshes::CubeMeshKey;
pub use model::*;
use tokio::sync::Semaphore;

use self::loading_material::{LoadingMaterialKey, LoadingShaderKey};
use anyhow::Context;

pub mod loading_material;

pub use ambient_ecs::generated::model::components::{
    model_animatable, model_from_url, model_loaded,
};

components!("model", {
    @[Networked, Store]
    animation_binder: HashMap<String, EntityId>,

    model: Arc<Model>,

    @[Networked, Store]
    pbr_renderer_primitives_from_url: Vec<PbrRenderPrimitiveFromUrl>,
    @[Networked, Store, MaybeResource]
    model_skins: Vec<ModelSkin>,
    @[Networked, Store]
    model_skin_ix: usize,

    @[Debuggable, Networked, Store]
    is_model_node: (),
});

#[tracing::instrument(skip(assets, async_run))]
async fn internal_spawn_models_from_defs(
    assets: &AssetCache,
    async_run: AsyncRun,
    entities_with_models: HashMap<String, Vec<EntityId>>,
) -> anyhow::Result<()> {
    // Meanwhile, spawn a spinning cube onto the entity.
    let cube = CubeMeshKey.get(assets);

    let mat = LoadingMaterialKey {
        speed: 2.0,
        scale: 6.0,
    }
    .get(assets);

    let cube = Entity::new()
        .with(
            primitives(),
            vec![RenderPrimitive {
                shader: cb(move |assets, config| {
                    StandardShaderKey {
                        material_shader: LoadingShaderKey.get(assets),
                        lit: false,
                        shadow_cascades: config.shadow_cascades,
                    }
                    .get(assets)
                }),
                material: mat,
                mesh: cube,
                lod: 0,
            }],
        )
        .with(gpu_primitives_mesh(), Default::default())
        .with(gpu_primitives_lod(), Default::default())
        .with(color(), vec4(0.0, 0.5, 1.0, 1.0))
        .with(main_scene(), ());

    let mut ids = entities_with_models
        .values()
        .flatten()
        .copied()
        .collect_vec();

    let cube_fail = Arc::new(cube.clone().with(color(), vec4(1.0, 0.0, 0.0, 1.0)));

    async_run.run(move |world| {
        ids.retain(|id| world.exists(*id));
        for id in ids {
            remove_model(world, id);
            tracing::debug!("Spawning cube model for {id}");
            log_result!(world.add_components(id, cube.clone()));

            // these should only be added if they do not already exist, as otherwise they will replace the existing values
            // TODO: consider backing up color, too
            for component in [local_to_world(), mesh_to_world()] {
                let _ = world.add_component_if_required(id, component, Default::default());
            }
        }
    });

    let iter = entities_with_models
        .into_iter()
        .map(|(url, ids)| async move {
            tracing::debug!("Loading model: {url:#?}");
            let mut url = match TypedAssetUrl::from_str(&url).context("Failed to parse url") {
                Ok(url) => url,
                Err(e) => return (ids, Err(e)),
            };
            if !url.0.path().contains("/models/") {
                url = match url
                    .0
                    .as_directory()
                    .join("models/main.json")
                    .context("Failed to join url")
                {
                    Ok(url) => url.into(),
                    Err(e) => return (ids, Err(e)),
                };
            };
            match ModelFromUrl(url)
                .get(assets)
                .await
                .context("Failed to load model")
            {
                Ok(v) => (ids, Ok(v)),
                Err(e) => (ids, Err(e)),
            }
        });

    let mut iter = futures::stream::iter(iter).buffer_unordered(4);
    while let Some((mut ids, model)) = iter.next().await {
        let cube_fail = cube_fail.clone();
        async_run.run(move |world| {
            // Remove the models which still exist
            ids.retain(|id| world.exists(*id));
            for id in &ids {
                remove_model(world, *id);
            }

            let len = ids.len();

            match model {
                Ok(model) => {
                    tracing::debug!("Spawning model: {:?} for {ids:?}", model.name());
                    let gpu = world.resource(gpu()).clone();
                    model.batch_spawn(
                        &gpu,
                        world,
                        &ModelSpawnOpts {
                            root: ModelSpawnRoot::AttachTo(ids),
                            // We need to keep the model alive on the entity here, or otherwise it'll unload from the asset store
                            root_components: Entity::new().with(self::model(), model.clone()),
                            ..Default::default()
                        },
                        len,
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to load model: {e:?}");
                    for id in ids {
                        remove_model(world, id);
                        tracing::debug!("Spawning cube model for {id}");
                        log_result!(world.add_components(id, (*cube_fail).clone()))
                    }
                }
            }
        })
    }
    Ok(())
}

pub fn model_systems() -> SystemGroup {
    SystemGroup::new(
        "model_systems",
        vec![
            query((children(),))
                .incl(model_from_url())
                .despawned()
                .to_system(|q, world, qs, _| {
                    for (_, (children,)) in q.collect_cloned(world, qs) {
                        for c in children {
                            if world.has_component(c, is_model_node()) {
                                despawn_recursive(world, c);
                            }
                        }
                    }
                }),
            query(())
                .incl(model_from_url())
                .despawned()
                .to_system(|q, world, qs, _| {
                    for (id, _) in q.collect_cloned(world, qs) {
                        remove_model(world, id);
                    }
                }),
            query((model_from_url().changed(),)).to_system(|q, world, qs, _| {
                let mut new_models = HashMap::<String, Vec<EntityId>>::new();
                for (id, (model_from_url,)) in q.iter(world, qs) {
                    let entry = new_models.entry(model_from_url.clone()).or_default();
                    entry.push(id);
                }
                if new_models.is_empty() {
                    return;
                }
                let assets = world.resource(asset_cache()).clone();
                let runtime = world.resource(runtime()).clone();
                let async_run = world.resource(async_run()).clone();

                runtime.spawn(async move {
                    internal_spawn_models_from_defs(&assets, async_run, new_models).await
                });
            }),
        ],
    )
}
fn remove_model(world: &mut World, entity: EntityId) {
    if let Ok(mut childs) = world.get_ref(entity, children()).map(|cs| cs.clone()) {
        childs.retain(|c| {
            if world.has_component(*c, is_model_node()) {
                despawn_recursive(world, *c);
                false
            } else {
                true
            }
        });
        world.set(entity, children(), childs).ok();
    }

    let mut components: Vec<ComponentDesc> = vec![
        primitives().desc(),
        gpu_primitives_mesh().desc(),
        gpu_primitives_lod().desc(),
        animation_binder().desc(),
        local_bounding_aabb().desc(),
        world_bounding_aabb().desc(),
        world_bounding_sphere().desc(),
        model_loaded().desc(),
    ];

    components.retain(|&comp| world.has_component_ref(entity, comp));
    world.remove_components(entity, components).ok();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFromUrl(pub TypedAssetUrl<ModelAssetType>);
impl ModelFromUrl {
    pub fn new(url: impl AsRef<str>) -> anyhow::Result<Self> {
        Ok(Self(TypedAssetUrl::from_str(url.as_ref())?))
    }
}
#[async_trait]
impl AsyncAssetKey<Result<Arc<Model>, AssetError>> for ModelFromUrl {
    async fn load(self, assets: AssetCache) -> Result<Arc<Model>, AssetError> {
        let url = self
            .0
            .clone()
            .abs()
            .context(format!("ModelFromUrl got relative url: {}", self.0))?;
        let data = BytesFromUrl::new(url.clone(), true).get(&assets).await?;
        let semaphore = ModelLoadSemaphore.get(&assets);
        let _permit = semaphore.acquire().await;
        let mut model = ambient_sys::task::block_in_place(|| Model::from_slice(&data))?;
        model.load(&assets, &url).await?;
        Ok(Arc::new(model))
    }
}

/// Limit the number of concurent model loads to 10
#[derive(Debug)]
struct ModelLoadSemaphore;
impl SyncAssetKey<Arc<Semaphore>> for ModelLoadSemaphore {
    fn load(&self, _assets: AssetCache) -> Arc<Semaphore> {
        Arc::new(Semaphore::new(10))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PbrRenderPrimitiveFromUrl {
    pub mesh: AssetUrl,
    pub material: Option<AssetUrl>,
    pub lod: usize,
}
impl PbrRenderPrimitiveFromUrl {
    pub fn resolve(
        &self,
        base_url: &AbsAssetUrl,
    ) -> anyhow::Result<PbrRenderPrimitiveFromResolvedUrl> {
        Ok(PbrRenderPrimitiveFromResolvedUrl {
            mesh: self.mesh.resolve(base_url)?,
            material: if let Some(x) = &self.material {
                Some(x.resolve(base_url)?)
            } else {
                None
            },
            lod: self.lod,
        })
    }
}
#[derive(Debug, Clone)]
pub struct PbrRenderPrimitiveFromResolvedUrl {
    pub mesh: AbsAssetUrl,
    pub material: Option<AbsAssetUrl>,
    pub lod: usize,
}
#[async_trait]
impl AsyncAssetKey<Result<Arc<RenderPrimitive>, AssetError>> for PbrRenderPrimitiveFromResolvedUrl {
    async fn load(self, assets: AssetCache) -> Result<Arc<RenderPrimitive>, AssetError> {
        let mesh = GpuMeshFromUrl {
            url: self.mesh,
            cache_on_disk: true,
        }
        .get(&assets)
        .await?;
        if let Some(mat_url) = self.material {
            let mat = PbrMaterialFromUrl(mat_url).get(&assets).await?;
            Ok(Arc::new(RenderPrimitive {
                material: mat.into(),
                shader: cb(get_pbr_shader),
                mesh,
                lod: self.lod,
            }))
        } else {
            Ok(Arc::new(RenderPrimitive {
                material: FlatMaterialKey::white().get(&assets),
                shader: cb(get_flat_shader),
                mesh,
                lod: self.lod,
            }))
        }
    }
}

pub fn bones_to_lines(world: &World, id: EntityId) -> Vec<Line> {
    fn inner(world: &World, id: EntityId, pos: Vec3, lines: &mut Vec<Line>) {
        let children = world.get_ref(id, children());
        for &c in children.into_iter().flatten() {
            let child_pos = get_world_position(world, c);
            if let Ok(child_pos) = child_pos {
                lines.push(Line(pos, child_pos));
                inner(world, c, child_pos, lines);
            }
        }
    }

    let pos = get_world_position(world, id);
    let mut lines = Vec::new();
    if let Ok(pos) = pos {
        inner(world, id, pos, &mut lines);
    }
    lines
}
