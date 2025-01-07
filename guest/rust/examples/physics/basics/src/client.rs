use ambient_api::prelude::*;
use packages::this::{assets, messages::Bonk};

pub mod packages;

#[main]
pub fn main() {
    let spatial_audio_player = audio::SpatialAudioPlayer::new();

    Bonk::subscribe(move |_ctx, data| {
        eprintln!("Bonk");
        spatial_audio_player.set_listener(data.listener);
        spatial_audio_player.play_sound_on_entity(assets::url("bonk.ogg"), data.emitter);
    });
}
