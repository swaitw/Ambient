use std::sync::Arc;

use ambient_audio::{
    hrtf::HrtfLib,
    track::{Track, TrackDecodeStream},
    Attenuation, AudioEmitter, AudioListener, AudioMixer, Sound, SoundId, Source, Spatial,
};
use ambient_ecs::{components, query, EntityId, Resource, World};
use ambient_element::ElementComponentExt;
use ambient_native_std::{asset_url::AbsAssetUrl, cb, Cb};
use ambient_ui_native::{
    graph::{Graph, GraphStyle},
    Editor, FlowColumn,
};
use anyhow::Context;
use derive_more::{Deref, DerefMut, From, Into};
use glam::{vec2, vec4};
use itertools::Itertools;
use parking_lot::Mutex;

use serde::{Deserialize, Serialize};

components!("audio", {
    @[Resource]
    hrtf_lib: Arc<HrtfLib>,
    audio_emitter: Arc<Mutex<AudioEmitter>>,
    audio_listener: Arc<Mutex<AudioListener>>,
    @[Resource]
    audio_sender: Arc<flume::Sender<AudioMessage>>,
    @[Resource]
    audio_mixer: AudioMixer,
    amplitude_arc: Arc<Mutex<f32>>,
    panning_arc: Arc<Mutex<f32>>,
    onepole_arc: Arc<Mutex<f32>>,
    looping_arc: Arc<Mutex<bool>>,
    sound_id: SoundId,
});

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioFx {
    Looping,
    Amplitude(f32),
    Panning(f32),
    OnePole(f32),
    LowPass(f32, f32),
    HighPass(f32, f32),
}

#[allow(clippy::large_enum_variant)]
pub enum AudioMessage {
    Track {
        track: Arc<Track>,
        url: AbsAssetUrl,
        fx: Vec<AudioFx>,
        uid: String,
    },
    Spatial(Spatial<TrackDecodeStream, Arc<Mutex<AudioListener>>, Arc<Mutex<AudioEmitter>>>),
    // sound id and amplitude
    UpdateVolume(String, f32),
    UpdatePanning(String, f32),
    AddOnePoleLpf(String, f32),
    StopById(String),
}

pub struct SoundInfo {
    pub url: AbsAssetUrl,
    pub id: SoundId,
    pub control_info: Vec<AudioControl>,
}

pub enum AudioControl {
    Amplitude(Arc<Mutex<f32>>),
    Panning(Arc<Mutex<f32>>),
    OnePole(Arc<Mutex<f32>>),
    LowPass(Arc<Mutex<f32>>, Arc<Mutex<f32>>),
    HighPass(Arc<Mutex<f32>>, Arc<Mutex<f32>>),
}

/// TODO: hook this into the Attenuation inside ambient_audio
#[derive(Serialize, Deserialize, Debug, Clone, Copy, DerefMut, Deref, From, Into)]
pub struct AttenuationEditorVisual(Attenuation);

impl Editor for AttenuationEditorVisual {
    fn editor(
        self,
        on_change: Cb<dyn Fn(Self) + Sync + Send>,
        opts: ambient_ui_native::EditorOpts,
    ) -> ambient_element::Element {
        let editor = Attenuation::editor(*self, cb(move |v| on_change(v.into())), opts);

        let x_max = self.inverse(0.01);

        const STEPS: u32 = 32;

        let points = (0..STEPS)
            .map(|v| {
                let x = (v as f32 / (STEPS - 1) as f32) * x_max;

                let y = self.attenuate(x).clamp(0.0, 2.0);

                vec2(x, y)
            })
            .collect_vec();

        let graph = Graph {
            points,
            style: GraphStyle {
                color: vec4(0.0, 0.0, 1.0, 1.0),
                ..Default::default()
            },
            width: 400.0,
            height: 200.0,
            ..Default::default()
        }
        .el();

        FlowColumn::el([editor, graph])
    }
}

pub fn get_audio_listener(world: &World) -> anyhow::Result<&Arc<Mutex<AudioListener>>> {
    let (_, listener) = query(audio_listener())
        .iter(world, None)
        .exactly_one()
        .map_err(|v| {
            anyhow::anyhow!(
                "Incorrect number of listeners in world. Additional: {:?}",
                v.count()
            )
        })?;

    Ok(listener)
}

/// Makes a sound source emit from the entity
pub fn play_sound_on_entity<S: 'static + Source>(
    world: &World,
    id: EntityId,
    source: S,
) -> anyhow::Result<Sound> {
    let hrtf_lib = world.resource(hrtf_lib());
    let mixer = world.resource(audio_mixer());
    let emitter = world
        .get_ref(id, audio_emitter())
        .context("No audio emitter on entity")?;

    let listener = get_audio_listener(world)?;

    Ok(mixer.play(source.spatial(hrtf_lib, listener.clone(), emitter.clone())))
}
