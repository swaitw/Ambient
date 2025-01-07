use std::time::Duration;

use ambient_api::{
    core::{
        app::components::name,
        layout::components::{fit_horizontal, fit_vertical, height, space_between_items, width},
        messages::Frame,
    },
    element::{use_query, use_state, use_state_with},
    entity::synchronized_resources,
    global::game_time,
    prelude::*,
};
use packages::this::{
    components::{bpm, track, track_audio_url, track_note_selection},
    messages::{Click, SetBpm},
};

mod common;

pub mod packages;

#[main]
pub fn main() {
    let mut cursor = 0;
    let mut last_note_time = game_time();
    let mut last_bpm = 0;
    let mut tree = Element::new().spawn_tree();
    Frame::subscribe(move |_| {
        let bpm = entity::get_component(synchronized_resources(), bpm()).unwrap_or_default();
        if bpm != last_bpm {
            cursor = 0;
            last_bpm = bpm;
        }

        let now = game_time();
        if now - last_note_time > Duration::from_secs_f32(seconds_per_note(bpm)) {
            last_note_time = now;
            cursor = (cursor + 1) % common::NOTE_COUNT;
            tree.migrate_root(&mut World, App::el(cursor));
        }
        tree.update(&mut World);
    });
}

#[element_component]
fn App(hooks: &mut Hooks, cursor: usize) -> Element {
    let mut tracks = use_query(hooks, (track(), track_note_selection()));
    tracks.sort_by_key(|t| t.1 .0);

    FlowColumn::el(
        std::iter::once(
            IntegerSlider {
                value: entity::get_component(synchronized_resources(), bpm()).unwrap_or_default()
                    as i32,
                on_change: Some(cb(|new_bpm| {
                    SetBpm::new(new_bpm as u32).send_server_reliable()
                })),
                min: 30,
                max: 300,
                width: 300.0,
                logarithmic: false,
                suffix: Some(" BPM"),
            }
            .el(),
        )
        .chain(
            tracks.into_iter().map(|(track_id, (_, track_selection))| {
                Track::el(track_id, track_selection, cursor)
            }),
        ),
    )
}

#[element_component]
fn Track(
    hooks: &mut Hooks,
    track_id: EntityId,
    track_selection: Vec<u32>,
    cursor: usize,
) -> Element {
    let track_name = entity::get_component(track_id, name()).unwrap_or_default();

    let (sound, _) = use_state_with(hooks, |_| {
        entity::get_component(track_id, track_audio_url()).unwrap()
    });

    let (audio_player, _) = use_state(hooks, audio::AudioPlayer::new());

    let (last_cursor, set_last_cursor) = use_state(hooks, 0);
    if cursor != last_cursor {
        if track_selection[cursor] != 0 {
            audio_player.play(sound);
        }
        set_last_cursor(cursor);
    }

    FlowColumn::el([
        Text::el(track_name),
        FlowRow::el(
            track_selection
                .iter()
                .enumerate()
                .map(|(index, &selected_hue)| {
                    let is_on_cursor = cursor == index;
                    Button::new(Note::el(selected_hue, is_on_cursor), move |_| {
                        Click::new(track_id, index as u32).send_server_reliable();
                    })
                    .style(ButtonStyle::Card)
                    .el()
                }),
        )
        .with_background(vec4(0.1, 0.1, 0.1, 1.))
        .with(fit_vertical(), Fit::Children)
        .with(fit_horizontal(), Fit::Children)
        .with_padding_even(10.)
        .with(space_between_items(), 2.),
    ])
}

#[element_component]
fn Note(_hooks: &mut Hooks, hue: u32, highlight: bool) -> Element {
    Rectangle
        .el()
        .with_background(match highlight {
            true => match hue == 0 {
                true => vec4(0.5, 0.5, 0.5, 1.),
                false => hsv_to_rgb(&[hue as f32, 0.7, 0.8]).extend(1.),
            },
            false => match hue == 0 {
                true => vec4(0.2, 0.2, 0.2, 1.),
                false => hsv_to_rgb(&[hue as f32, 0.7, 1.0]).extend(1.),
            },
        })
        .with(width(), 50.)
        .with(height(), 50.)
}

fn seconds_per_note(bpm: u32) -> f32 {
    let seconds_per_beat = 60.0 / (bpm as f32).max(1.0);
    seconds_per_beat / 4.0
}

fn hsv_to_rgb([h, s, v]: &[f32; 3]) -> Vec3 {
    let c = v * s;
    let p = (h / 60.) % 6.;
    let x = c * (1.0 - ((p % 2.) - 1.).abs());
    let m = Vec3::ONE * (v - c);

    m + match p.trunc() as i32 {
        0 => vec3(c, x, 0.),
        1 => vec3(x, c, 0.),
        2 => vec3(0., c, x),
        3 => vec3(0., x, c),
        4 => vec3(x, 0., c),
        5 => vec3(c, 0., x),
        _ => vec3(0., 0., 0.),
    }
}
