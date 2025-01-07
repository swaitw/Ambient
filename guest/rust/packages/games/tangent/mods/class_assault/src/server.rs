use ambient_api::prelude::*;

use packages::tangent_schema::concepts::{CharacterDef, PlayerClass};

pub mod packages;

#[main]
pub fn main() {
    PlayerClass {
        is_class: (),

        name: "Assault".to_string(),
        description: "A versatile choice for those who seek balance in speed, firepower, and maneuverability.".to_string(),
        icon_url: packages::this::assets::url("icon.png"),
        def_ref: CharacterDef {
            max_health: 100.0,
            model_url: packages::this::assets::url("Ch29_nonPBR.fbx"),
            speed: 0.06,
            run_speed_multiplier: 1.5,
            strafe_speed_multiplier: 0.8,
        }
        .spawn(),
    }
    .spawn();
}
