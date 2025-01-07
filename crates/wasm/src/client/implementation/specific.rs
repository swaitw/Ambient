//! Used to implement all the client-specific host functions.
//!
//! If implementing a trait that is also available on the server, it should go in [super].

use std::sync::Arc;

use ambient_core::{
    async_ecs::async_run,
    gpu,
    player::local_user_id,
    runtime,
    window::{window_ctl, WindowCtl},
};
use ambient_ecs::generated::input::messages::ClipboardGet;
use ambient_gpu::texture::Texture;
use ambient_input::{player_prev_raw_input, player_raw_input};
use ambient_native_std::mesh::MeshBuilder;
use ambient_network::client::client_state;
use ambient_procedurals::{
    new_material_handle, new_mesh_handle, new_sampler_handle, new_texture_handle,
    procedural_storage,
};
use ambient_renderer::pbr_material::{PbrMaterialConfig, PbrMaterialParams};

use anyhow::Context;
use glam::Vec4;
use wgpu::TextureViewDescriptor;
use winit::window::CursorGrabMode;

use super::Bindings;
use crate::shared::{
    conversion::{FromBindgen, IntoBindgen},
    implementation::message,
    message::{MessageExt, Target},
    wit,
};

use ambient_core::camera::{clip_position_to_world_ray, world_to_clip_space};

impl wit::client_message::Host for Bindings {
    fn send(
        &mut self,
        target: wit::client_message::Target,
        name: String,
        data: Vec<u8>,
    ) -> anyhow::Result<()> {
        use wit::client_message::Target as WitTarget;

        let module_id = self.id;
        let world = self.world_mut();

        match target {
            WitTarget::ServerUnreliable | WitTarget::ServerReliable => {
                let connection = world
                    .resource(client_state())
                    .as_ref()
                    .context("no game client")?
                    .transport
                    .clone();

                message::send_networked(
                    world,
                    connection,
                    module_id,
                    &name,
                    &data,
                    matches!(target, WitTarget::ServerReliable),
                )
            }
            WitTarget::LocalBroadcast(include_self) => {
                message::send_local(world, module_id, Target::All { include_self }, name, data)
            }
            WitTarget::Local(id) => message::send_local(
                world,
                module_id,
                Target::PackageOrModule(id.from_bindgen()),
                name,
                data,
            ),
        }
    }
}
impl wit::client_player::Host for Bindings {
    fn get_local(&mut self) -> anyhow::Result<wit::types::EntityId> {
        crate::shared::implementation::player::get_by_user_id(
            self.world(),
            self.world().resource(local_user_id()).clone(),
        )
        .transpose()
        .unwrap()
    }
}
impl wit::client_input::Host for Bindings {
    fn get(&mut self) -> anyhow::Result<wit::client_input::Input> {
        Ok(self
            .world()
            .resource(player_raw_input())
            .clone()
            .into_bindgen())
    }

    fn get_previous(&mut self) -> anyhow::Result<wit::client_input::Input> {
        Ok(self
            .world()
            .resource(player_prev_raw_input())
            .clone()
            .into_bindgen())
    }

    fn set_cursor(&mut self, icon: wit::client_input::CursorIcon) -> anyhow::Result<()> {
        Ok(self
            .world()
            .resource(ambient_core::window::window_ctl())
            .send(ambient_core::window::WindowCtl::SetCursorIcon(
                icon.from_bindgen().into(),
            ))?)
    }

    fn set_cursor_visible(&mut self, visible: bool) -> anyhow::Result<()> {
        Ok(self
            .world()
            .resource(ambient_core::window::window_ctl())
            .send(ambient_core::window::WindowCtl::ShowCursor(visible))?)
    }

    fn set_cursor_lock(&mut self, lock: bool) -> anyhow::Result<()> {
        let grab_mode = if lock {
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            {
                CursorGrabMode::Confined
            }

            #[cfg(target_os = "macos")]
            {
                CursorGrabMode::Locked
            }

            #[cfg(target_os = "unknown")]
            {
                CursorGrabMode::Locked
            }
        } else {
            CursorGrabMode::None
        };

        Ok(self
            .world()
            .resource(ambient_core::window::window_ctl())
            .send(ambient_core::window::WindowCtl::GrabCursor(grab_mode))?)
    }
}
impl wit::client_camera::Host for Bindings {
    fn clip_position_to_world_ray(
        &mut self,
        camera: wit::types::EntityId,
        clip_space_pos: wit::types::Vec2,
    ) -> anyhow::Result<wit::types::Ray> {
        let mut ray = clip_position_to_world_ray(
            self.world(),
            camera.from_bindgen(),
            clip_space_pos.from_bindgen(),
        )?;
        ray.dir *= -1.;
        Ok(ray.into_bindgen())
    }

    fn screen_to_clip_space(
        &mut self,
        screen_pos: wit::types::Vec2,
    ) -> anyhow::Result<wit::types::Vec2> {
        Ok(
            ambient_core::window::screen_to_clip_space(self.world(), screen_pos.from_bindgen())
                .into_bindgen(),
        )
    }

    fn screen_position_to_world_ray(
        &mut self,
        camera: wit::types::EntityId,
        screen_pos: wit::types::Vec2,
    ) -> anyhow::Result<wit::types::Ray> {
        let clip_space =
            ambient_core::window::screen_to_clip_space(self.world(), screen_pos.from_bindgen());
        let mut ray = clip_position_to_world_ray(self.world(), camera.from_bindgen(), clip_space)?;
        ray.dir *= -1.;
        Ok(ray.into_bindgen())
    }

    fn world_to_screen(
        &mut self,
        camera: wit::types::EntityId,
        world_pos: wit::types::Vec3,
    ) -> anyhow::Result<wit::types::Vec3> {
        let clip_pos = world_to_clip_space(
            self.world(),
            camera.from_bindgen(),
            world_pos.from_bindgen(),
        )?;
        Ok(ambient_core::window::clip_to_screen_space(self.world(), clip_pos).into_bindgen())
    }
}

impl wit::client_clipboard::Host for Bindings {
    fn get(&mut self) -> anyhow::Result<()> {
        let module_id = self.id;
        let async_run = self.world().resource(async_run()).clone();
        let runtime = self.world().resource(runtime());
        let task = async move {
            let contents = ambient_sys::clipboard::get().await;
            async_run.run(move |world| {
                ClipboardGet { contents }
                    .send(world, Some(module_id))
                    .unwrap();
            });
        };

        #[cfg(target_os = "unknown")]
        runtime.spawn_local(task);
        #[cfg(not(target_os = "unknown"))]
        runtime.spawn(task);

        Ok(())
    }

    fn set(&mut self, text: String) -> anyhow::Result<()> {
        ambient_sys::clipboard::set_background(text, |res| {
            if let Err(err) = res {
                tracing::error!("Failed to set clipboard: {:?}", err);
            }
        });

        Ok(())
    }
}

impl wit::client_window::Host for Bindings {
    fn set_fullscreen(&mut self, fullscreen: bool) -> anyhow::Result<()> {
        self.world_mut()
            .resource(window_ctl())
            .send(WindowCtl::SetFullscreen(fullscreen))?;
        Ok(())
    }
}

impl wit::client_mesh::Host for Bindings {
    fn create(
        &mut self,
        desc: wit::client_mesh::Descriptor,
    ) -> anyhow::Result<wit::client_mesh::Handle> {
        let wit::client_mesh::Descriptor { vertices, indices } = desc;
        let mut positions = Vec::with_capacity(vertices.len());
        let mut normals = Vec::with_capacity(vertices.len());
        let mut tangents = Vec::with_capacity(vertices.len());
        let mut texcoords = Vec::with_capacity(vertices.len());
        for v in &vertices {
            positions.push(v.position.from_bindgen());
            normals.push(v.normal.from_bindgen());
            tangents.push(v.tangent.from_bindgen());
            texcoords.push(v.texcoord0.from_bindgen());
        }
        let mesh = MeshBuilder {
            positions,
            normals,
            tangents,
            texcoords: vec![texcoords],
            indices,
            ..MeshBuilder::default()
        }
        .build()?;

        let world = self.world_mut();
        let storage = world.resource_mut(procedural_storage());
        let mesh_handle = new_mesh_handle();
        storage.meshes.insert(mesh_handle, mesh);
        Ok(mesh_handle.into_bindgen())
    }
    fn destroy(&mut self, handle: wit::client_mesh::Handle) -> anyhow::Result<()> {
        let world = self.world_mut();
        let storage = world.resource_mut(procedural_storage());
        storage.meshes.remove(handle.from_bindgen());
        Ok(())
    }
}
impl wit::client_texture::Host for Bindings {
    fn create2d(
        &mut self,
        desc: wit::client_texture::Descriptor2d,
    ) -> anyhow::Result<wit::client_texture::Handle> {
        let world = self.world_mut();
        let gpu = world.resource(gpu());
        let texture = Texture::new_with_data(
            gpu,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: desc.width,
                    height: desc.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: desc.format.from_bindgen(),
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            &desc.data,
        );
        let texture = Arc::new(texture);
        let texture_view = Arc::new(texture.create_view(&TextureViewDescriptor::default()));
        let storage = world.resource_mut(procedural_storage());
        let texture_handle = new_texture_handle();
        storage.textures.insert(texture_handle, texture_view);
        Ok(texture_handle.into_bindgen())
    }
    fn destroy(&mut self, handle: wit::client_texture::Handle) -> anyhow::Result<()> {
        let world = self.world_mut();
        let storage = world.resource_mut(procedural_storage());
        storage.textures.remove(handle.from_bindgen());
        Ok(())
    }
}
impl wit::client_sampler::Host for Bindings {
    fn create(
        &mut self,
        desc: wit::client_sampler::Descriptor,
    ) -> anyhow::Result<wit::client_sampler::Handle> {
        let world = self.world_mut();
        let gpu = world.resource(gpu());
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: desc.address_mode_u.from_bindgen(),
            address_mode_v: desc.address_mode_v.from_bindgen(),
            address_mode_w: desc.address_mode_w.from_bindgen(),
            mag_filter: desc.mag_filter.from_bindgen(),
            min_filter: desc.min_filter.from_bindgen(),
            mipmap_filter: desc.mipmap_filter.from_bindgen(),
            ..wgpu::SamplerDescriptor::default()
        });
        let sampler = Arc::new(sampler);
        let storage = world.resource_mut(procedural_storage());
        let sampler_handle = new_sampler_handle();
        storage.samplers.insert(sampler_handle, sampler);
        Ok(sampler_handle.into_bindgen())
    }
    fn destroy(&mut self, handle: wit::client_sampler::Handle) -> anyhow::Result<()> {
        let world = self.world_mut();
        let storage = world.resource_mut(procedural_storage());
        storage.samplers.remove(handle.from_bindgen());
        Ok(())
    }
}
impl wit::client_material::Host for Bindings {
    fn create(
        &mut self,
        desc: wit::client_material::Descriptor,
    ) -> anyhow::Result<wit::client_material::Handle> {
        let world = self.world_mut();
        let storage = world.resource_mut(procedural_storage());
        let material = PbrMaterialConfig {
            source: "Procedural Material".to_string(),
            name: "Procedural Material".to_string(),
            params: PbrMaterialParams {
                base_color_factor: Vec4::ONE,
                emissive_factor: Vec4::ZERO,
                alpha_cutoff: 0.0,
                ..PbrMaterialParams::default()
            },
            base_color: Arc::clone(storage.textures.get(desc.base_color_map.from_bindgen())),
            normalmap: Arc::clone(storage.textures.get(desc.normal_map.from_bindgen())),
            metallic_roughness: Arc::clone(
                storage
                    .textures
                    .get(desc.metallic_roughness_map.from_bindgen()),
            ),
            sampler: Arc::clone(storage.samplers.get(desc.sampler.from_bindgen())),
            transparent: Some(desc.transparent),
            double_sided: None,
            depth_write_enabled: None,
        };
        let material_handle = new_material_handle();
        storage.materials.insert(material_handle, material);
        Ok(material_handle.into_bindgen())
    }
    fn destroy(&mut self, handle: wit::client_material::Handle) -> anyhow::Result<()> {
        let world = self.world_mut();
        let storage = world.resource_mut(procedural_storage());
        storage.materials.remove(handle.from_bindgen());
        Ok(())
    }
}
