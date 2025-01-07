use std::f32::consts::PI;

use ambient_api::{
    core::{
        messages::Frame,
        physics::components::linear_velocity,
        player::components::{is_player, user_id},
        primitives::{components::cube, concepts::Sphere},
        rendering::components::color,
        transform::components::*,
    },
    prelude::*,
};
use packages::this::{
    assets,
    components::{player_movement_direction, track_audio_url},
    messages::{Input, Ping},
};

mod constants;
use constants::*;

pub mod packages;

#[main]
pub fn main() {
    let bgm_url = assets::url("Kevin_MacLeod_8bit_Dungeon_Boss_ncs.ogg");

    entity::add_component(entity::synchronized_resources(), track_audio_url(), bgm_url);
    // Spawn field, paddles and ball
    Entity::new()
        .with(cube(), ())
        .with(scale(), vec3(X_BOUNDARY * 2., Y_BOUNDARY * 2., 1.))
        .with(translation(), vec3(0., 0., 1.))
        .with(color(), vec4(1., 1., 1., 1.))
        .spawn();
    let paddles = [
        spawn_paddle(true, vec3(255., 0., 0.)),
        spawn_paddle(false, vec3(0., 255., 0.)),
    ];
    let ball = Sphere::suggested()
        .make()
        .with(scale(), vec3(BALL_RADIUS, BALL_RADIUS, 1.))
        .with(translation(), vec3(0., 0., -1.))
        .with(color(), vec4(255., 255., 255., 1.))
        .spawn();

    // When a player spawns, create a camera and other components for them
    spawn_query(is_player()).bind(move |players| {
        for (player, _) in players {
            entity::add_component(player, player_movement_direction(), 0.0);
        }
    });

    // When a player despawns, clean up their objects
    let player_objects_query = query(user_id()).build();
    despawn_query(user_id()).requires(is_player()).bind({
        move |players| {
            let player_objects = player_objects_query.evaluate();
            for (_, player_user_id) in &players {
                for (id, _) in player_objects
                    .iter()
                    .filter(|(_, object_user_id)| *player_user_id == *object_user_id)
                {
                    entity::despawn(*id);
                }
            }
        }
    });

    // Ball movement
    query((linear_velocity(), translation())).each_frame(move |balls| {
        for (id, (velocity, position)) in balls {
            let new_position = position + velocity;
            entity::set_component(id, translation(), new_position);
            if new_position.y.abs() > Y_BOUNDARY - BALL_RADIUS / 2. {
                // bounce from top and bottom "walls"
                Ping::new().send_client_broadcast_reliable();
                let new_velocity = vec3(velocity.x, -velocity.y, velocity.z);
                entity::set_component(id, linear_velocity(), new_velocity);
            }
        }
    });

    Input::subscribe(|ctx, msg| {
        let Some(player_id) = ctx.client_entity_id() else {
            return;
        };

        entity::set_component(player_id, player_movement_direction(), msg.direction);
    });

    Frame::subscribe(move |_| {
        let players = entity::get_all(is_player());

        // start the ball if we have 2 players and ball has no velocity
        if players.len() >= 2 && entity::get_component(ball, linear_velocity()).is_none() {
            entity::add_component(ball, linear_velocity(), gen_ball_velocity());
        }

        // handle players' input
        for (i, player) in players.into_iter().enumerate() {
            let paddle = paddles[i % 2];
            let Some(direction) = entity::get_component(player, player_movement_direction()) else {
                continue;
            };
            let Some(mut paddle_position) = entity::get_component(paddle, translation()) else {
                continue;
            };

            paddle_position.y += direction * PADDLE_V_PER_FRAME;
            paddle_position.y = paddle_position.y.clamp(
                PADDLE_LENGTH / 2. - Y_BOUNDARY,
                Y_BOUNDARY - PADDLE_LENGTH / 2.,
            );
            entity::set_component(paddle, translation(), paddle_position);
        }

        // paddle bouncing
        if let Some(ball_position) = entity::get_component(ball, translation()) {
            if ball_position.x.abs() > X_BOUNDARY - BALL_RADIUS / 2. {
                let paddle = paddles[(ball_position.x.signum() + 1.) as usize / 2];
                let paddle_position = entity::get_component(paddle, translation()).unwrap();
                if let Some(velocity) = entity::get_component(ball, linear_velocity()) {
                    let new_velocity = if (paddle_position.y - ball_position.y).abs()
                        < PADDLE_LENGTH / 2. + BALL_RADIUS / 2.
                    {
                        // bounce from the paddle
                        Ping::new().send_client_broadcast_reliable();
                        // accelerate a bit
                        let new_v_len = (velocity.x.powi(2) + velocity.y.powi(2)).sqrt()
                            * (1. + BALL_ACCELERATION);
                        // adjust the angle to allow for spinning depending on which part of the paddle was hit by the ball
                        let ratio_from_center = (paddle_position.y - ball_position.y)
                            / PADDLE_LENGTH
                            * paddle_position.x.signum();
                        let new_v_angle =
                            velocity.y.atan2(-velocity.x) + BALL_SPINNING * ratio_from_center;
                        vec3(
                            new_v_angle.cos() * new_v_len,
                            new_v_angle.sin() * new_v_len,
                            velocity.z,
                        )
                    } else {
                        // ball passed the paddle

                        // place it back in the center
                        entity::set_component(ball, translation(), Vec3::ZERO);
                        let mut v = gen_ball_velocity();
                        // make it go against the losing player (keep the sign on x)
                        v.x *= velocity.x.signum();
                        v
                    };
                    entity::set_component(ball, linear_velocity(), new_velocity);
                }
            }
        }
    });
}

fn spawn_paddle(left: bool, paddle_color: Vec3) -> EntityId {
    let x = X_BOUNDARY + PADDLE_WIDTH / 2.;
    Entity::new()
        .with(cube(), ())
        .with(scale(), vec3(PADDLE_WIDTH, PADDLE_LENGTH, 1.))
        .with(translation(), vec3(if left { -x } else { x }, 0., 0.))
        .with(color(), paddle_color.extend(1.))
        .spawn()
}

fn gen_ball_velocity() -> Vec3 {
    let angle = random::<f32>() * (PI / 5.) + PI / 10.;
    let y_sign = if random::<bool>() { 1. } else { -1. };
    vec3(
        angle.cos() * BALL_V_PER_FRAME,
        angle.sin() * BALL_V_PER_FRAME * y_sign,
        0.,
    )
}
