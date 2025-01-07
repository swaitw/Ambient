use ambient_api::{
    core::{
        audio::components::{amplitude, panning, stop_now},
        layout::components::space_between_items,
    },
    element::{use_frame, use_state},
    prelude::*,
};
use packages::this::assets;

pub mod packages;

#[main]
pub fn main() {
    let audio_player = audio::AudioPlayer::new();
    audio_player.set_looping(true); // try false
    App::el(audio_player).spawn_interactive();
}

#[element_component]
fn App(hooks: &mut Hooks, audio_player: audio::AudioPlayer) -> Element {
    let (f32_value, set_f32_value) = use_state(hooks, 100.);
    let (sound, set_sound) = use_state(hooks, None);
    let (pan, set_pan) = use_state(hooks, 0.);
    use_frame(hooks, {
        let set_sound = set_sound.clone();
        move |_world| {
            if let Some(s) = sound {
                if !entity::exists(s) {
                    set_sound(None);
                }
            }
        }
    });
    FlowColumn::el([
        Text::el("Amplitude:"),
        Slider {
            value: f32_value,
            on_change: Some(cb({
                let audio_player = audio_player.clone();
                move |v| {
                    set_f32_value(v);
                    audio_player.set_amplitude(v / 100.);
                    if let Some(s) = sound {
                        if entity::exists(s) {
                            entity::add_component(s, amplitude(), v / 100.);
                        }
                    }
                }
            })),
            min: 0.0,
            max: 100.,
            width: 100.,
            logarithmic: false,
            round: Some(2),
            suffix: Some("%"),
        }
        .el(),
        Text::el("Panning:"),
        Slider {
            value: pan,
            on_change: Some(cb({
                let audio_player = audio_player.clone();
                move |v| {
                    set_pan(v);
                    audio_player.set_panning(v);
                    if let Some(s) = sound {
                        if entity::exists(s) {
                            entity::add_component(s, panning(), v);
                        }
                    }
                }
            })),
            min: -1.0,
            max: 1.0,
            width: 100.,
            logarithmic: false,
            round: Some(4),
            suffix: None,
        }
        .el(),
        Button::new("play sound", {
            let set_sound = set_sound.clone();
            move |_| {
                let id = audio_player.play(assets::url("amen_break.wav"));
                // mono ogg
                // let id = audio_player.play(
                //     assets::url("455516__ispeakwaves__the-plan-upbeat-loop-no-voice-edit-mono-track.ogg")
                // );
                set_sound(Some(id));
            }
        })
        .disabled(sound.is_some())
        .toggled(true)
        .el(),
        Button::new("stop sound", {
            move |_| {
                if let Some(s) = sound {
                    if entity::exists(s) {
                        entity::add_component(s, stop_now(), ());
                        set_sound(None);
                    } else {
                        set_sound(None);
                    }
                }
            }
        })
        .disabled(sound.is_none())
        .toggled(true)
        .el(),
    ])
    .with(space_between_items(), STREET)
    .with_padding_even(STREET)
}
