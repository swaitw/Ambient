#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused)]
use std::io::Read;

use ambient_package_rt::message_serde::{MessageSerde, MessageSerdeError};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub use raw::ambient_core::*;

#[allow(
    unused,
    clippy::unit_arg,
    clippy::let_and_return,
    clippy::approx_constant,
    clippy::unused_unit
)]
mod raw {
    pub mod ambient_core {
        pub mod animation {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("animation" , { # [doc = "**Is animation player**: This entity is treated as an animation player. Attach an animation node as a child for it to play.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Is animation player"] , Description ["This entity is treated as an animation player. Attach an animation node as a child for it to play."]] is_animation_player : () , # [doc = "**Animation errors**: A list of errors that were produced trying to play the animation.\n\n*Attributes*: Debuggable"] @ [Debuggable , Name ["Animation errors"] , Description ["A list of errors that were produced trying to play the animation."]] animation_errors : Vec :: < String > , # [doc = "**Apply animation player**: Apply the designated animation player to this entity and its sub-tree.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Apply animation player"] , Description ["Apply the designated animation player to this entity and its sub-tree."]] apply_animation_player : EntityId , # [doc = "**Play clip from URL**: Make this entity a 'play animation clip' node. The value is the URL to the clip we'd like to play.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Play clip from URL"] , Description ["Make this entity a 'play animation clip' node. The value is the URL to the clip we'd like to play."]] play_clip_from_url : String , # [doc = "**Looping**: When this is true, the animation clip will repeat infinitely.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Looping"] , Description ["When this is true, the animation clip will repeat infinitely."]] looping : bool , # [doc = "**Speed**: Animation playback speed. Default is 1, higher values speeds up the animation.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Speed"] , Description ["Animation playback speed. Default is 1, higher values speeds up the animation."]] speed : f32 , # [doc = "**Start time**: Start time of an animation node.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Start time"] , Description ["Start time of an animation node."]] start_time : Duration , # [doc = "**Freeze at percentage**: Sample the input animation at a certain percentage of the animation track length.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Freeze at percentage"] , Description ["Sample the input animation at a certain percentage of the animation track length."]] freeze_at_percentage : f32 , # [doc = "**Freeze at time**: Sample the input animation at a certain time (in seconds).\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Freeze at time"] , Description ["Sample the input animation at a certain time (in seconds)."]] freeze_at_time : f32 , # [doc = "**Clip duration**: The clip duration is loaded from the clip, and then applied to the entity.\n\n*Attributes*: Debuggable"] @ [Debuggable , Name ["Clip duration"] , Description ["The clip duration is loaded from the clip, and then applied to the entity."]] clip_duration : f32 , # [doc = "**Clip loaded**: The clip has been loaded.\n\n*Attributes*: Debuggable"] @ [Debuggable , Name ["Clip loaded"] , Description ["The clip has been loaded."]] clip_loaded : () , # [doc = "**Clip load error**: There was an error loading the clip.\n\n*Attributes*: Debuggable"] @ [Debuggable , Name ["Clip load error"] , Description ["There was an error loading the clip."]] clip_load_error : String , # [doc = "**Blend**: Blend two animations together. The values is the blend weight. Use `children` to set the animations. Blend 0 means we only sample from the first animation, 1 means only the second one, and values in between blend between them.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Blend"] , Description ["Blend two animations together. The values is the blend weight. Use `children` to set the animations. Blend 0 means we only sample from the first animation, 1 means only the second one, and values in between blend between them."]] blend : f32 , # [doc = "**Mask bind ids**: List of bind ids that will be masked.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Mask bind ids"] , Description ["List of bind ids that will be masked."]] mask_bind_ids : Vec :: < String > , # [doc = "**Mask weights**: Weights for each bind id in `mask_bind_ids`.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Mask weights"] , Description ["Weights for each bind id in `mask_bind_ids`."]] mask_weights : Vec :: < f32 > , # [doc = "**Retarget Model from URL**: Retarget the animation using the model at the given URL.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Retarget Model from URL"] , Description ["Retarget the animation using the model at the given URL."]] retarget_model_from_url : String , # [doc = "**Retarget animation scaled**: Retarget animation scaled. True means normalize hip.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Retarget animation scaled"] , Description ["Retarget animation scaled. True means normalize hip."]] retarget_animation_scaled : bool , # [doc = "**Apply base pose**: Apply the base pose to this clip.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Apply base pose"] , Description ["Apply the base pose to this clip."]] apply_base_pose : () , # [doc = "**Bind id**: Animation bind ID.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Bind id"] , Description ["Animation bind ID."]] bind_id : String , # [doc = "**Bind ids**: Animation bind IDs.\n\n*Attributes*: Debuggable, Store"] @ [Debuggable , Store , Name ["Bind ids"] , Description ["Animation bind IDs."]] bind_ids : Vec :: < String > , });
            }
        }
        pub mod app {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("app" , { # [doc = "**Cursor position**: Absolute mouse cursor position in screen-space. This is the *logical* position. Multiply by the `window_scale_factor` to get the physical position.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Cursor position"] , Description ["Absolute mouse cursor position in screen-space. This is the *logical* position. Multiply by the `window_scale_factor` to get the physical position."]] cursor_position : Vec2 , # [doc = "**Delta time**: How long the previous tick took in seconds.\n\n*Attributes*: Debuggable, Resource"] @ [Debuggable , Resource , Name ["Delta time"] , Description ["How long the previous tick took in seconds."]] delta_time : f32 , # [doc = "**Epoch time**: Time since epoch (Jan 1, 1970). Non_monotonic.\n\n*Attributes*: Debuggable, Resource"] @ [Debuggable , Resource , Name ["Epoch time"] , Description ["Time since epoch (Jan 1, 1970). Non_monotonic."]] epoch_time : Duration , # [doc = "**Game time**: Time since the game was started. Monotonic.\n\n*Attributes*: Debuggable, Resource"] @ [Debuggable , Resource , Name ["Game time"] , Description ["Time since the game was started. Monotonic."]] game_time : Duration , # [doc = "**Element**: The identifier of the `Element` that controls this entity.\n\nThis is automatically generated by `ElementTree`.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Element"] , Description ["The identifier of the `Element` that controls this entity.\nThis is automatically generated by `ElementTree`."]] element : String , # [doc = "**Element unmanaged children**: If this is set, the user is expected to manage the children of the `Element` themselves.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Element unmanaged children"] , Description ["If this is set, the user is expected to manage the children of the `Element` themselves."]] element_unmanaged_children : () , # [doc = "**Main scene**: If attached, this entity belongs to the main scene.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Main scene"] , Description ["If attached, this entity belongs to the main scene."]] main_scene : () , # [doc = "**Map seed**: A random number seed for this map.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Map seed"] , Description ["A random number seed for this map."]] map_seed : u64 , # [doc = "**Name**: A human-friendly name for this entity.\n\n*Attributes*: Debuggable, Networked, Store, MaybeResource"] @ [Debuggable , Networked , Store , MaybeResource , Name ["Name"] , Description ["A human-friendly name for this entity."]] name : String , # [doc = "**Description**: A human-friendly description for this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Description"] , Description ["A human-friendly description for this entity."]] description : String , # [doc = "**Main Package Name**: The name of the main package being run.\n\nDefaults to \"Ambient\".\n\n*Attributes*: Debuggable, Resource"] @ [Debuggable , Resource , Name ["Main Package Name"] , Description ["The name of the main package being run.\nDefaults to \"Ambient\"."]] main_package_name : String , # [doc = "**Selectable**: If attached, this object can be selected in the editor.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Selectable"] , Description ["If attached, this object can be selected in the editor."]] selectable : () , # [doc = "**Snap to ground**: This object should automatically be moved with the terrain if the terrain is changed.\n\nThe value is the offset from the terrain.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Snap to ground"] , Description ["This object should automatically be moved with the terrain if the terrain is changed.\nThe value is the offset from the terrain."]] snap_to_ground : f32 , # [doc = "**Tags**: Tags for categorizing this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Tags"] , Description ["Tags for categorizing this entity."]] tags : Vec :: < String > , # [doc = "**UI scene**: If attached, this entity belongs to the UI scene.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["UI scene"] , Description ["If attached, this entity belongs to the UI scene."]] ui_scene : () , # [doc = "**Window logical size**: The logical size is the physical size divided by the scale factor.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Window logical size"] , Description ["The logical size is the physical size divided by the scale factor."]] window_logical_size : UVec2 , # [doc = "**Window physical size**: The physical size is the actual number of pixels on the screen.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Window physical size"] , Description ["The physical size is the actual number of pixels on the screen."]] window_physical_size : UVec2 , # [doc = "**Window scale factor**: The DPI/pixel scale factor of the window.\n\nOn standard displays, this is 1, but it can be higher on high-DPI displays like Apple Retina displays.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Window scale factor"] , Description ["The DPI/pixel scale factor of the window.\nOn standard displays, this is 1, but it can be higher on high-DPI displays like Apple Retina displays."]] window_scale_factor : f64 , # [doc = "**Reference count**: Ref-counted enity. If this entity doesn't have a `parent` component, and the ref count reaches 0, it will be removed together with all its children recursively.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Reference count"] , Description ["Ref-counted enity. If this entity doesn't have a `parent` component, and the ref count reaches 0, it will be removed together with all its children recursively."]] ref_count : u32 , });
            }
        }
        pub mod audio {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("audio" , { # [doc = "**Is audio player**: The entity is an audio player.\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Is audio player"] , Description ["The entity is an audio player."]] is_audio_player : () , # [doc = "**Is spatial audio player**: The entity is a spatial audio player.\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Is spatial audio player"] , Description ["The entity is a spatial audio player."]] is_spatial_audio_player : () , # [doc = "**Spatial audio emitter**: The entity is a spatial audio emitter.\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Spatial audio emitter"] , Description ["The entity is a spatial audio emitter."]] spatial_audio_emitter : EntityId , # [doc = "**Spatial audio listener**: The entity is a spatial audio listener.\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Spatial audio listener"] , Description ["The entity is a spatial audio listener."]] spatial_audio_listener : EntityId , # [doc = "**Looping**: Whether or not the audio should loop.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Looping"] , Description ["Whether or not the audio should loop.\n"]] looping : bool , # [doc = "**One pole low pass filter**: With this component, the audio will be filtered with a one pole low pass filter.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["One pole low pass filter"] , Description ["With this component, the audio will be filtered with a one pole low pass filter.\n"]] onepole_lpf : f32 , # [doc = "**Playing sound**: The entity with this comp is a playing sound.\n\nWe can attach other components to it to control the sound parameters.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Playing sound"] , Description ["The entity with this comp is a playing sound.\nWe can attach other components to it to control the sound parameters.\n"]] playing_sound : () , # [doc = "**Amplitude**: The amplitude of the audio.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Amplitude"] , Description ["The amplitude of the audio.\n"]] amplitude : f32 , # [doc = "**Panning**: The panning of the audio.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Panning"] , Description ["The panning of the audio.\n"]] panning : f32 , # [doc = "**Low_pass filter**: Low pass filter. The first value is the cutoff frequency, the second is the bandwidth.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Low_pass filter"] , Description ["Low pass filter. The first value is the cutoff frequency, the second is the bandwidth.\n"]] lpf : Vec2 , # [doc = "**High_pass filter**: High pass filter. The first value is the cutoff frequency, the second is the bandwidth.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["High_pass filter"] , Description ["High pass filter. The first value is the cutoff frequency, the second is the bandwidth.\n"]] hpf : Vec2 , # [doc = "**Audio URL**: The URL of the assets.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Audio URL"] , Description ["The URL of the assets.\n"]] audio_url : String , # [doc = "**Trigger at this frame**: The system will watch for this component and PLAY the audio at this frame,\n\nusing the other components as parameters.\n\nThen set it back to false.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Trigger at this frame"] , Description ["The system will watch for this component and PLAY the audio at this frame,\nusing the other components as parameters.\nThen set it back to false.\n"]] play_now : () , # [doc = "**Stop at this frame**: The system will watch for this component and STOP the audio at this frame,\n\nusing the other components as parameters.\n\nThen set it back to false.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Stop at this frame"] , Description ["The system will watch for this component and STOP the audio at this frame,\nusing the other components as parameters.\nThen set it back to false.\n"]] stop_now : () , });
            }
        }
        pub mod camera {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("camera" , { # [doc = "**Active camera**: The camera with the highest `active_camera` value will be used for rendering. Cameras are also filtered by the `user_id`.\n\nIf there's no `user_id`, the camera is considered global and potentially applies to all users (if its `active_camera` value is high enough).\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Active camera"] , Description ["The camera with the highest `active_camera` value will be used for rendering. Cameras are also filtered by the `user_id`.\nIf there's no `user_id`, the camera is considered global and potentially applies to all users (if its `active_camera` value is high enough)."]] active_camera : f32 , # [doc = "**Aspect ratio**: The aspect ratio of this camera.\n\nIf `aspect_ratio_from_window` is set, this will be automatically updated to match the window.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Aspect ratio"] , Description ["The aspect ratio of this camera.\nIf `aspect_ratio_from_window` is set, this will be automatically updated to match the window."]] aspect_ratio : f32 , # [doc = "**Aspect ratio from window**: If attached, the `aspect_ratio` component will be automatically updated to match the aspect ratio of the window. Should point to an entity with a `window_physical_size` component.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Aspect ratio from window"] , Description ["If attached, the `aspect_ratio` component will be automatically updated to match the aspect ratio of the window. Should point to an entity with a `window_physical_size` component."]] aspect_ratio_from_window : EntityId , # [doc = "**Far plane**: The far plane of this camera, measured in meters.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Far plane"] , Description ["The far plane of this camera, measured in meters."]] far : f32 , # [doc = "**Fog**: If attached, this camera will see/render fog.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Fog"] , Description ["If attached, this camera will see/render fog."]] fog : () , # [doc = "**Field of View Y**: The field of view of this camera in the Y/vertical direction, measured in radians.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Field of View Y"] , Description ["The field of view of this camera in the Y/vertical direction, measured in radians."]] fovy : f32 , # [doc = "**Near plane**: The near plane of this camera, measured in meters.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Near plane"] , Description ["The near plane of this camera, measured in meters."]] near : f32 , # [doc = "**Orthographic projection**: If attached, this camera will use a standard orthographic projection matrix.\n\nEnsure that the `orthographic_` components are set, including `left`, right`, `top` and `bottom`, as well as `near` and `far`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Orthographic projection"] , Description ["If attached, this camera will use a standard orthographic projection matrix.\nEnsure that the `orthographic_` components are set, including `left`, right`, `top` and `bottom`, as well as `near` and `far`."]] orthographic : () , # [doc = "**Orthographic bottom**: The bottom bound for this `orthographic` camera.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Orthographic bottom"] , Description ["The bottom bound for this `orthographic` camera."]] orthographic_bottom : f32 , # [doc = "**Orthographic from window**: The bounds of this orthographic camera will be updated to match the window automatically. Should point to an entity with a `window_logical_size` component.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Orthographic from window"] , Description ["The bounds of this orthographic camera will be updated to match the window automatically. Should point to an entity with a `window_logical_size` component."]] orthographic_from_window : EntityId , # [doc = "**Orthographic left**: The left bound for this `orthographic` camera.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Orthographic left"] , Description ["The left bound for this `orthographic` camera."]] orthographic_left : f32 , # [doc = "**Orthographic right**: The right bound for this `orthographic` camera.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Orthographic right"] , Description ["The right bound for this `orthographic` camera."]] orthographic_right : f32 , # [doc = "**Orthographic top**: The top bound for this `orthographic` camera.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Orthographic top"] , Description ["The top bound for this `orthographic` camera."]] orthographic_top : f32 , # [doc = "**Perspective projection**: If attached, this camera will use a standard perspective projection matrix.\n\nEnsure that `near` and `far` are set.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Perspective projection"] , Description ["If attached, this camera will use a standard perspective projection matrix.\nEnsure that `near` and `far` are set."]] perspective : () , # [doc = "**Perspective-infinite-reverse projection**: If attached, this camera will use a perspective-infinite-reverse projection matrix.\n\nThis is well-suited for rendering large worlds as it has no far plane. Ensure `near` is set.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Perspective-infinite-reverse projection"] , Description ["If attached, this camera will use a perspective-infinite-reverse projection matrix.\nThis is well-suited for rendering large worlds as it has no far plane. Ensure `near` is set."]] perspective_infinite_reverse : () , # [doc = "**Projection**: The projection matrix of this camera.\n\nThis can be driven by other components, including `perspective` and `perspective_infinite_reverse`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Projection"] , Description ["The projection matrix of this camera.\nThis can be driven by other components, including `perspective` and `perspective_infinite_reverse`."]] projection : Mat4 , # [doc = "**Projection-view**: The composition of the projection and view (inverse-local-to-world) matrices.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Projection-view"] , Description ["The composition of the projection and view (inverse-local-to-world) matrices."]] projection_view : Mat4 , # [doc = "**Shadows far plane**: The far plane for the shadow camera, measured in meters.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Shadows far plane"] , Description ["The far plane for the shadow camera, measured in meters."]] shadows_far : f32 , });
            }
        }
        pub mod ecs {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("ecs" , { # [doc = "**Don't automatically despawn on module unload**: Indicates that this entity shouldn't be despawned when the module that spawned it unloads.\n\n*Attributes*: Debuggable, Store"] @ [Debuggable , Store , Name ["Don't automatically despawn on module unload"] , Description ["Indicates that this entity shouldn't be despawned when the module that spawned it unloads."]] dont_despawn_on_unload : () , # [doc = "**Don't store**: Indicates that this entity shouldn't be stored on disk.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Don't store"] , Description ["Indicates that this entity shouldn't be stored on disk."]] dont_store : () , # [doc = "**ID**: The ID of the entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["ID"] , Description ["The ID of the entity."]] id : EntityId , # [doc = "**Remove at game time**: If attached, this entity will be despawned at the specified game time.\n\n*Attributes*: Debuggable"] @ [Debuggable , Name ["Remove at game time"] , Description ["If attached, this entity will be despawned at the specified game time."]] remove_at_game_time : Duration , });
            }
        }
        pub mod hierarchy {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("hierarchy" , { # [doc = "**Parent**: The parent of this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Parent"] , Description ["The parent of this entity."]] parent : EntityId , # [doc = "**Children**: The children of this entity.\n\n*Attributes*: Debuggable, Store, MaybeResource"] @ [Debuggable , Store , MaybeResource , Name ["Children"] , Description ["The children of this entity."]] children : Vec :: < EntityId > , # [doc = "**Unmanaged children**: This children component is not updated automatically for this entity when this component is attached.\n\n*Attributes*: Debuggable, Networked, Store, MaybeResource"] @ [Debuggable , Networked , Store , MaybeResource , Name ["Unmanaged children"] , Description ["This children component is not updated automatically for this entity when this component is attached."]] unmanaged_children : () , });
            }
        }
        pub mod input {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("input" , { # [doc = "**Mouse over entity**: The entity the mouse is currently over.\n\n*Attributes*: Debuggable, Resource"] @ [Debuggable , Resource , Name ["Mouse over entity"] , Description ["The entity the mouse is currently over."]] mouse_over_entity : EntityId , # [doc = "**Mouse over distance**: This distance to the entity that the mouse is currently over.\n\n*Attributes*: Debuggable, Resource"] @ [Debuggable , Resource , Name ["Mouse over distance"] , Description ["This distance to the entity that the mouse is currently over."]] mouse_over_distance : f32 , # [doc = "**Mouse over**: The number of mouse cursors that are currently over this entity.\n\n*Attributes*: Debuggable"] @ [Debuggable , Name ["Mouse over"] , Description ["The number of mouse cursors that are currently over this entity."]] is_mouse_over : u32 , # [doc = "**Mouse pickable max**: This entity can be clicked by the mouse, and this component defines the max AABB bound of the click area.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Mouse pickable max"] , Description ["This entity can be clicked by the mouse, and this component defines the max AABB bound of the click area."]] mouse_pickable_max : Vec3 , # [doc = "**Mouse pickable min**: This entity can be clicked by the mouse, and this component defines the min AABB bound of the click area.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Mouse pickable min"] , Description ["This entity can be clicked by the mouse, and this component defines the min AABB bound of the click area."]] mouse_pickable_min : Vec3 , });
            }
            #[doc = r" Auto-generated message definitions. Messages are used to communicate with the runtime, the other side of the network,"]
            #[doc = r" and with other modules."]
            pub mod messages {
                use crate::{Entity, EntityId};
                use ambient_package_rt::message_serde::{
                    Message, MessageSerde, MessageSerdeError, ModuleMessage, RuntimeMessage,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                #[derive(Clone, Debug)]
                #[doc = "**MouseOverChanged**: Mouse over has been updated"]
                pub struct MouseOverChanged {
                    pub from_external: bool,
                    pub mouse_over: EntityId,
                    pub distance: f32,
                }
                impl MouseOverChanged {
                    #[allow(clippy::too_many_arguments)]
                    pub fn new(
                        from_external: impl Into<bool>,
                        mouse_over: impl Into<EntityId>,
                        distance: impl Into<f32>,
                    ) -> Self {
                        Self {
                            from_external: from_external.into(),
                            mouse_over: mouse_over.into(),
                            distance: distance.into(),
                        }
                    }
                }
                impl Message for MouseOverChanged {
                    fn id() -> &'static str {
                        "ambient_core::input::MouseOverChanged"
                    }
                    fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                        let mut output = vec![];
                        self.from_external.serialize_message_part(&mut output)?;
                        self.mouse_over.serialize_message_part(&mut output)?;
                        self.distance.serialize_message_part(&mut output)?;
                        Ok(output)
                    }
                    fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                        Ok(Self {
                            from_external: bool::deserialize_message_part(&mut input)?,
                            mouse_over: EntityId::deserialize_message_part(&mut input)?,
                            distance: f32::deserialize_message_part(&mut input)?,
                        })
                    }
                }
                impl ModuleMessage for MouseOverChanged {}
                #[derive(Clone, Debug)]
                #[doc = "**ClipboardGet**: Sent to a package that has requested the clipboard contents."]
                pub struct ClipboardGet {
                    pub contents: Option<String>,
                }
                impl ClipboardGet {
                    #[allow(clippy::too_many_arguments)]
                    pub fn new(contents: impl Into<Option<String>>) -> Self {
                        Self {
                            contents: contents.into(),
                        }
                    }
                }
                impl Message for ClipboardGet {
                    fn id() -> &'static str {
                        "ambient_core::input::ClipboardGet"
                    }
                    fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                        let mut output = vec![];
                        self.contents.serialize_message_part(&mut output)?;
                        Ok(output)
                    }
                    fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                        Ok(Self {
                            contents: Option::<String>::deserialize_message_part(&mut input)?,
                        })
                    }
                }
                impl RuntimeMessage for ClipboardGet {}
            }
        }
        pub mod layout {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("layout" , { # [doc = "**Align horizontal**: Layout alignment: horizontal.\n\n*Attributes*: Debuggable, Networked, Store, Enum"] @ [Debuggable , Networked , Store , Enum , Name ["Align horizontal"] , Description ["Layout alignment: horizontal."]] align_horizontal : crate :: generated :: raw :: ambient_core :: layout :: types :: Align , # [doc = "**Align vertical**: Layout alignment: vertical.\n\n*Attributes*: Debuggable, Networked, Store, Enum"] @ [Debuggable , Networked , Store , Enum , Name ["Align vertical"] , Description ["Layout alignment: vertical."]] align_vertical : crate :: generated :: raw :: ambient_core :: layout :: types :: Align , # [doc = "**Docking**: Layout docking.\n\n*Attributes*: Debuggable, Networked, Store, Enum"] @ [Debuggable , Networked , Store , Enum , Name ["Docking"] , Description ["Layout docking."]] docking : crate :: generated :: raw :: ambient_core :: layout :: types :: Docking , # [doc = "**Fit horizontal**: Layout fit: horizontal.\n\n*Attributes*: Debuggable, Networked, Store, Enum"] @ [Debuggable , Networked , Store , Enum , Name ["Fit horizontal"] , Description ["Layout fit: horizontal."]] fit_horizontal : crate :: generated :: raw :: ambient_core :: layout :: types :: Fit , # [doc = "**Fit vertical**: Layout fit: vertical.\n\n*Attributes*: Debuggable, Networked, Store, Enum"] @ [Debuggable , Networked , Store , Enum , Name ["Fit vertical"] , Description ["Layout fit: vertical."]] fit_vertical : crate :: generated :: raw :: ambient_core :: layout :: types :: Fit , # [doc = "**Layout**: Layout.\n\n*Attributes*: Debuggable, Networked, Store, Enum"] @ [Debuggable , Networked , Store , Enum , Name ["Layout"] , Description ["Layout."]] layout : crate :: generated :: raw :: ambient_core :: layout :: types :: Layout , # [doc = "**Orientation**: Layout orientation.\n\n*Attributes*: Debuggable, Networked, Store, Enum"] @ [Debuggable , Networked , Store , Enum , Name ["Orientation"] , Description ["Layout orientation."]] orientation : crate :: generated :: raw :: ambient_core :: layout :: types :: Orientation , # [doc = "**Is book file**: This is a file in a `layout_bookcase`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Is book file"] , Description ["This is a file in a `layout_bookcase`."]] is_book_file : () , # [doc = "**Margin**: Layout margin: [top, right, bottom, left].\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Margin"] , Description ["Layout margin: [top, right, bottom, left]."]] margin : Vec4 , # [doc = "**Padding**: Layout padding: [top, right, bottom, left].\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Padding"] , Description ["Layout padding: [top, right, bottom, left]."]] padding : Vec4 , # [doc = "**Mesh to local from size**: Update the `mesh_to_local` based on the width and height of this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Mesh to local from size"] , Description ["Update the `mesh_to_local` based on the width and height of this entity."]] mesh_to_local_from_size : () , # [doc = "**Minimum height**: The minimum height of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Minimum height"] , Description ["The minimum height of a UI element."]] min_height : f32 , # [doc = "**Minimum width**: The minimum width of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Minimum width"] , Description ["The minimum width of a UI element."]] min_width : f32 , # [doc = "**Maximum height**: The maximum height of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Maximum height"] , Description ["The maximum height of a UI element."]] max_height : f32 , # [doc = "**Maximum width**: The maximum width of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Maximum width"] , Description ["The maximum width of a UI element."]] max_width : f32 , # [doc = "**Is screen**: This entity will be treated as a screen. Used by the Screen ui component.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Is screen"] , Description ["This entity will be treated as a screen. Used by the Screen ui component."]] is_screen : () , # [doc = "**Space between items**: Space between items in a layout.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Space between items"] , Description ["Space between items in a layout."]] space_between_items : f32 , # [doc = "**Width**: The width of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Width"] , Description ["The width of a UI element."]] width : f32 , # [doc = "**Height**: The height of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Height"] , Description ["The height of a UI element."]] height : f32 , # [doc = "**GPU UI size**: Upload the width and height of this UI element to the GPU.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["GPU UI size"] , Description ["Upload the width and height of this UI element to the GPU."]] gpu_ui_size : Vec4 , });
            }
            #[doc = r" Auto-generated type definitions."]
            pub mod types {
                use ambient_package_rt::message_serde::*;
                use serde;
                #[derive(
                    Copy,
                    Clone,
                    Debug,
                    PartialEq,
                    Eq,
                    serde :: Serialize,
                    serde :: Deserialize,
                    Default,
                )]
                #[serde(crate = "self::serde")]
                #[doc = "**Align**: Layout alignment."]
                pub enum Align {
                    #[default]
                    #[doc = "Begin"]
                    Begin,
                    #[doc = "Center"]
                    Center,
                    #[doc = "End"]
                    End,
                }
                impl crate::EnumComponent for Align {
                    fn to_u32(&self) -> u32 {
                        match self {
                            Self::Begin => Align::Begin as u32,
                            Self::Center => Align::Center as u32,
                            Self::End => Align::End as u32,
                        }
                    }
                    fn from_u32(value: u32) -> Option<Self> {
                        if value == Align::Begin as u32 {
                            return Some(Self::Begin);
                        }
                        if value == Align::Center as u32 {
                            return Some(Self::Center);
                        }
                        if value == Align::End as u32 {
                            return Some(Self::End);
                        }
                        None
                    }
                }
                impl MessageSerde for Align {
                    fn serialize_message_part(
                        &self,
                        output: &mut Vec<u8>,
                    ) -> Result<(), MessageSerdeError> {
                        crate::EnumComponent::to_u32(self).serialize_message_part(output)
                    }
                    fn deserialize_message_part(
                        input: &mut dyn std::io::Read,
                    ) -> Result<Self, MessageSerdeError> {
                        crate::EnumComponent::from_u32(u32::deserialize_message_part(input)?)
                            .ok_or(MessageSerdeError::InvalidValue)
                    }
                }
                #[derive(
                    Copy,
                    Clone,
                    Debug,
                    PartialEq,
                    Eq,
                    serde :: Serialize,
                    serde :: Deserialize,
                    Default,
                )]
                #[serde(crate = "self::serde")]
                #[doc = "**Fit**: Layout fit."]
                pub enum Fit {
                    #[default]
                    #[doc = "None"]
                    None,
                    #[doc = "Parent"]
                    Parent,
                    #[doc = "Children"]
                    Children,
                }
                impl crate::EnumComponent for Fit {
                    fn to_u32(&self) -> u32 {
                        match self {
                            Self::None => Fit::None as u32,
                            Self::Parent => Fit::Parent as u32,
                            Self::Children => Fit::Children as u32,
                        }
                    }
                    fn from_u32(value: u32) -> Option<Self> {
                        if value == Fit::None as u32 {
                            return Some(Self::None);
                        }
                        if value == Fit::Parent as u32 {
                            return Some(Self::Parent);
                        }
                        if value == Fit::Children as u32 {
                            return Some(Self::Children);
                        }
                        None
                    }
                }
                impl MessageSerde for Fit {
                    fn serialize_message_part(
                        &self,
                        output: &mut Vec<u8>,
                    ) -> Result<(), MessageSerdeError> {
                        crate::EnumComponent::to_u32(self).serialize_message_part(output)
                    }
                    fn deserialize_message_part(
                        input: &mut dyn std::io::Read,
                    ) -> Result<Self, MessageSerdeError> {
                        crate::EnumComponent::from_u32(u32::deserialize_message_part(input)?)
                            .ok_or(MessageSerdeError::InvalidValue)
                    }
                }
                #[derive(
                    Copy,
                    Clone,
                    Debug,
                    PartialEq,
                    Eq,
                    serde :: Serialize,
                    serde :: Deserialize,
                    Default,
                )]
                #[serde(crate = "self::serde")]
                #[doc = "**Orientation**: Layout orientation."]
                pub enum Orientation {
                    #[default]
                    #[doc = "Horizontal"]
                    Horizontal,
                    #[doc = "Vertical"]
                    Vertical,
                }
                impl crate::EnumComponent for Orientation {
                    fn to_u32(&self) -> u32 {
                        match self {
                            Self::Horizontal => Orientation::Horizontal as u32,
                            Self::Vertical => Orientation::Vertical as u32,
                        }
                    }
                    fn from_u32(value: u32) -> Option<Self> {
                        if value == Orientation::Horizontal as u32 {
                            return Some(Self::Horizontal);
                        }
                        if value == Orientation::Vertical as u32 {
                            return Some(Self::Vertical);
                        }
                        None
                    }
                }
                impl MessageSerde for Orientation {
                    fn serialize_message_part(
                        &self,
                        output: &mut Vec<u8>,
                    ) -> Result<(), MessageSerdeError> {
                        crate::EnumComponent::to_u32(self).serialize_message_part(output)
                    }
                    fn deserialize_message_part(
                        input: &mut dyn std::io::Read,
                    ) -> Result<Self, MessageSerdeError> {
                        crate::EnumComponent::from_u32(u32::deserialize_message_part(input)?)
                            .ok_or(MessageSerdeError::InvalidValue)
                    }
                }
                #[derive(
                    Copy,
                    Clone,
                    Debug,
                    PartialEq,
                    Eq,
                    serde :: Serialize,
                    serde :: Deserialize,
                    Default,
                )]
                #[serde(crate = "self::serde")]
                #[doc = "**Docking**: The edge to dock to."]
                pub enum Docking {
                    #[default]
                    #[doc = "Left"]
                    Left,
                    #[doc = "Right"]
                    Right,
                    #[doc = "Top"]
                    Top,
                    #[doc = "Bottom"]
                    Bottom,
                    #[doc = "Fill"]
                    Fill,
                }
                impl crate::EnumComponent for Docking {
                    fn to_u32(&self) -> u32 {
                        match self {
                            Self::Left => Docking::Left as u32,
                            Self::Right => Docking::Right as u32,
                            Self::Top => Docking::Top as u32,
                            Self::Bottom => Docking::Bottom as u32,
                            Self::Fill => Docking::Fill as u32,
                        }
                    }
                    fn from_u32(value: u32) -> Option<Self> {
                        if value == Docking::Left as u32 {
                            return Some(Self::Left);
                        }
                        if value == Docking::Right as u32 {
                            return Some(Self::Right);
                        }
                        if value == Docking::Top as u32 {
                            return Some(Self::Top);
                        }
                        if value == Docking::Bottom as u32 {
                            return Some(Self::Bottom);
                        }
                        if value == Docking::Fill as u32 {
                            return Some(Self::Fill);
                        }
                        None
                    }
                }
                impl MessageSerde for Docking {
                    fn serialize_message_part(
                        &self,
                        output: &mut Vec<u8>,
                    ) -> Result<(), MessageSerdeError> {
                        crate::EnumComponent::to_u32(self).serialize_message_part(output)
                    }
                    fn deserialize_message_part(
                        input: &mut dyn std::io::Read,
                    ) -> Result<Self, MessageSerdeError> {
                        crate::EnumComponent::from_u32(u32::deserialize_message_part(input)?)
                            .ok_or(MessageSerdeError::InvalidValue)
                    }
                }
                #[derive(
                    Copy,
                    Clone,
                    Debug,
                    PartialEq,
                    Eq,
                    serde :: Serialize,
                    serde :: Deserialize,
                    Default,
                )]
                #[serde(crate = "self::serde")]
                #[doc = "**Layout**: The type of the layout to use."]
                pub enum Layout {
                    #[default]
                    #[doc = "Bottom-up flow layout."]
                    Flow,
                    #[doc = "Top-down dock layout."]
                    Dock,
                    #[doc = "Min-max bookcase layout."]
                    Bookcase,
                    #[doc = "Width to children."]
                    WidthToChildren,
                }
                impl crate::EnumComponent for Layout {
                    fn to_u32(&self) -> u32 {
                        match self {
                            Self::Flow => Layout::Flow as u32,
                            Self::Dock => Layout::Dock as u32,
                            Self::Bookcase => Layout::Bookcase as u32,
                            Self::WidthToChildren => Layout::WidthToChildren as u32,
                        }
                    }
                    fn from_u32(value: u32) -> Option<Self> {
                        if value == Layout::Flow as u32 {
                            return Some(Self::Flow);
                        }
                        if value == Layout::Dock as u32 {
                            return Some(Self::Dock);
                        }
                        if value == Layout::Bookcase as u32 {
                            return Some(Self::Bookcase);
                        }
                        if value == Layout::WidthToChildren as u32 {
                            return Some(Self::WidthToChildren);
                        }
                        None
                    }
                }
                impl MessageSerde for Layout {
                    fn serialize_message_part(
                        &self,
                        output: &mut Vec<u8>,
                    ) -> Result<(), MessageSerdeError> {
                        crate::EnumComponent::to_u32(self).serialize_message_part(output)
                    }
                    fn deserialize_message_part(
                        input: &mut dyn std::io::Read,
                    ) -> Result<Self, MessageSerdeError> {
                        crate::EnumComponent::from_u32(u32::deserialize_message_part(input)?)
                            .ok_or(MessageSerdeError::InvalidValue)
                    }
                }
            }
        }
        pub mod model {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("model" , { # [doc = "**Model animatable**: Controls whether this model can be animated.\n\n*Attributes*: MaybeResource, Debuggable, Networked, Store"] @ [MaybeResource , Debuggable , Networked , Store , Name ["Model animatable"] , Description ["Controls whether this model can be animated."]] model_animatable : bool , # [doc = "**Model from URL**: Load a model from the given URL or relative path.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Model from URL"] , Description ["Load a model from the given URL or relative path."]] model_from_url : String , # [doc = "**Model loaded**: If attached, this entity has a model attached to it.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Model loaded"] , Description ["If attached, this entity has a model attached to it."]] model_loaded : () , });
            }
        }
        pub mod network {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("network" , { # [doc = "**Is remote entity**: If attached, this entity was not spawned locally (e.g. if this is the client, it was spawned by the server).\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Is remote entity"] , Description ["If attached, this entity was not spawned locally (e.g. if this is the client, it was spawned by the server)."]] is_remote_entity : () , # [doc = "**Is persistent resources**: If attached, this entity contains global resources that are persisted to disk and synchronized to clients.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Is persistent resources"] , Description ["If attached, this entity contains global resources that are persisted to disk and synchronized to clients."]] is_persistent_resources : () , # [doc = "**Is synchronized resources**: If attached, this entity contains global resources that are synchronized to clients, but not persisted.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Is synchronized resources"] , Description ["If attached, this entity contains global resources that are synchronized to clients, but not persisted."]] is_synced_resources : () , # [doc = "**No sync**: If attached, this entity will not be synchronized to clients.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["No sync"] , Description ["If attached, this entity will not be synchronized to clients."]] no_sync : () , });
            }
        }
        pub mod package {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("package" , { # [doc = "**Main Package ID**: The ID of the main package.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Main Package ID"] , Description ["The ID of the main package."]] main_package_id : EntityId , # [doc = "**Is Package**: Whether or not this entity is a package.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Is Package"] , Description ["Whether or not this entity is a package."]] is_package : () , # [doc = "**Enabled**: Whether or not this package is enabled.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Enabled"] , Description ["Whether or not this package is enabled."]] enabled : bool , # [doc = "**ID**: The ID of the package.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["ID"] , Description ["The ID of the package."]] id : String , # [doc = "**Name**: The name of the package.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Name"] , Description ["The name of the package."]] name : String , # [doc = "**Version**: The version of the package.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Version"] , Description ["The version of the package."]] version : String , # [doc = "**Authors**: The authors of the package.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Authors"] , Description ["The authors of the package."]] authors : Vec :: < String > , # [doc = "**Description**: The description of the package. If not attached, the package does not have a description.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Description"] , Description ["The description of the package. If not attached, the package does not have a description."]] description : String , # [doc = "**Repository**: The repository of the package. If not attached, the package does not have a repository.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Repository"] , Description ["The repository of the package. If not attached, the package does not have a repository."]] repository : String , # [doc = "**For Playables**: The playable IDs that this package is for. This package must be a `Mod`.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["For Playables"] , Description ["The playable IDs that this package is for. This package must be a `Mod`."]] for_playables : Vec :: < String > , # [doc = "**Asset URL**: The asset URL (i.e. where the built assets are) of the package.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Asset URL"] , Description ["The asset URL (i.e. where the built assets are) of the package."]] asset_url : String , # [doc = "**Client Modules**: The clientside WASM modules spawned by this package.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Client Modules"] , Description ["The clientside WASM modules spawned by this package."]] client_modules : Vec :: < EntityId > , # [doc = "**Server Modules**: The serverside WASM modules spawned by this package.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Server Modules"] , Description ["The serverside WASM modules spawned by this package."]] server_modules : Vec :: < EntityId > , });
            }
            #[doc = r" Auto-generated message definitions. Messages are used to communicate with the runtime, the other side of the network,"]
            #[doc = r" and with other modules."]
            pub mod messages {
                use crate::{Entity, EntityId};
                use ambient_package_rt::message_serde::{
                    Message, MessageSerde, MessageSerdeError, ModuleMessage, RuntimeMessage,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                #[derive(Clone, Debug)]
                #[doc = "**PackageLoadSuccess**: A package has successfully loaded. Note that this may fire before all of its constituent WASM modules have loaded."]
                pub struct PackageLoadSuccess {
                    pub package: EntityId,
                    pub url: String,
                }
                impl PackageLoadSuccess {
                    #[allow(clippy::too_many_arguments)]
                    pub fn new(package: impl Into<EntityId>, url: impl Into<String>) -> Self {
                        Self {
                            package: package.into(),
                            url: url.into(),
                        }
                    }
                }
                impl Message for PackageLoadSuccess {
                    fn id() -> &'static str {
                        "ambient_core::package::PackageLoadSuccess"
                    }
                    fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                        let mut output = vec![];
                        self.package.serialize_message_part(&mut output)?;
                        self.url.serialize_message_part(&mut output)?;
                        Ok(output)
                    }
                    fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                        Ok(Self {
                            package: EntityId::deserialize_message_part(&mut input)?,
                            url: String::deserialize_message_part(&mut input)?,
                        })
                    }
                }
                impl RuntimeMessage for PackageLoadSuccess {}
                #[derive(Clone, Debug)]
                #[doc = "**PackageLoadFailure**: A package has failed to load."]
                pub struct PackageLoadFailure {
                    pub url: String,
                    pub reason: String,
                }
                impl PackageLoadFailure {
                    #[allow(clippy::too_many_arguments)]
                    pub fn new(url: impl Into<String>, reason: impl Into<String>) -> Self {
                        Self {
                            url: url.into(),
                            reason: reason.into(),
                        }
                    }
                }
                impl Message for PackageLoadFailure {
                    fn id() -> &'static str {
                        "ambient_core::package::PackageLoadFailure"
                    }
                    fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                        let mut output = vec![];
                        self.url.serialize_message_part(&mut output)?;
                        self.reason.serialize_message_part(&mut output)?;
                        Ok(output)
                    }
                    fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                        Ok(Self {
                            url: String::deserialize_message_part(&mut input)?,
                            reason: String::deserialize_message_part(&mut input)?,
                        })
                    }
                }
                impl RuntimeMessage for PackageLoadFailure {}
            }
            #[doc = r" Auto-generated type definitions."]
            pub mod types {
                use ambient_package_rt::message_serde::*;
                use serde;
                #[derive(
                    Copy,
                    Clone,
                    Debug,
                    PartialEq,
                    Eq,
                    serde :: Serialize,
                    serde :: Deserialize,
                    Default,
                )]
                #[serde(crate = "self::serde")]
                #[doc = "**PackageContent**: The content type of the package."]
                pub enum PackageContent {
                    #[default]
                    #[doc = "A playable experience."]
                    Playable,
                    #[doc = "An asset."]
                    Asset,
                    #[doc = "A tool."]
                    Tool,
                    #[doc = "A mod."]
                    Mod,
                }
                impl crate::EnumComponent for PackageContent {
                    fn to_u32(&self) -> u32 {
                        match self {
                            Self::Playable => PackageContent::Playable as u32,
                            Self::Asset => PackageContent::Asset as u32,
                            Self::Tool => PackageContent::Tool as u32,
                            Self::Mod => PackageContent::Mod as u32,
                        }
                    }
                    fn from_u32(value: u32) -> Option<Self> {
                        if value == PackageContent::Playable as u32 {
                            return Some(Self::Playable);
                        }
                        if value == PackageContent::Asset as u32 {
                            return Some(Self::Asset);
                        }
                        if value == PackageContent::Tool as u32 {
                            return Some(Self::Tool);
                        }
                        if value == PackageContent::Mod as u32 {
                            return Some(Self::Mod);
                        }
                        None
                    }
                }
                impl MessageSerde for PackageContent {
                    fn serialize_message_part(
                        &self,
                        output: &mut Vec<u8>,
                    ) -> Result<(), MessageSerdeError> {
                        crate::EnumComponent::to_u32(self).serialize_message_part(output)
                    }
                    fn deserialize_message_part(
                        input: &mut dyn std::io::Read,
                    ) -> Result<Self, MessageSerdeError> {
                        crate::EnumComponent::from_u32(u32::deserialize_message_part(input)?)
                            .ok_or(MessageSerdeError::InvalidValue)
                    }
                }
            }
        }
        pub mod physics {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("physics" , { # [doc = "**Angular velocity**: Angular velocity (radians/second) of this entity in the physics scene.\n\nUpdating this component will update the entity's angular velocity in the physics scene.\n\n\n\nNote that changing this component will forcibly set the velocity; changing the velocity every frame may lead to unexpected behavior, like improper physics or collisions failing.\n\n\n\nIf you need to adjust the velocity each frame, consider applying an impulse using `physics` functions instead.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Angular velocity"] , Description ["Angular velocity (radians/second) of this entity in the physics scene.\nUpdating this component will update the entity's angular velocity in the physics scene.\n\nNote that changing this component will forcibly set the velocity; changing the velocity every frame may lead to unexpected behavior, like improper physics or collisions failing.\n\nIf you need to adjust the velocity each frame, consider applying an impulse using `physics` functions instead."]] angular_velocity : Vec3 , # [doc = "**Cube collider**: If attached, this entity will have a cube physics collider.\n\n`x, y, z` is the size of the cube.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Cube collider"] , Description ["If attached, this entity will have a cube physics collider.\n`x, y, z` is the size of the cube."]] cube_collider : Vec3 , # [doc = "**Character controller height**: The height of the physics character controller attached to this entity.\n\nIf an entity has both this and a `character_controller_radius`, it will be given a physical character collider.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Character controller height"] , Description ["The height of the physics character controller attached to this entity.\nIf an entity has both this and a `character_controller_radius`, it will be given a physical character collider."]] character_controller_height : f32 , # [doc = "**Character controller radius**: The radius of the physics character controller attached to this entity.\n\nIf an entity has both this and a `character_controller_height`, it will be given a physical character collider.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Character controller radius"] , Description ["The radius of the physics character controller attached to this entity.\nIf an entity has both this and a `character_controller_height`, it will be given a physical character collider."]] character_controller_radius : f32 , # [doc = "**Collider from URL**: This entity will load its physics collider from the URL.\n\nThe value is the URL to load from.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Collider from URL"] , Description ["This entity will load its physics collider from the URL.\nThe value is the URL to load from."]] collider_from_url : String , # [doc = "**Collider loaded**: This component is automatically attached to an entity once the collider has been loaded (through e.g. `collider_from_url`).\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Collider loaded"] , Description ["This component is automatically attached to an entity once the collider has been loaded (through e.g. `collider_from_url`)."]] collider_loaded : () , # [doc = "**Collider loads**: Contains all colliders that were loaded in this physics tick.\n\n*Attributes*: Debuggable, Networked, Resource, Store"] @ [Debuggable , Networked , Resource , Store , Name ["Collider loads"] , Description ["Contains all colliders that were loaded in this physics tick."]] collider_loads : Vec :: < EntityId > , # [doc = "**Contact offset**: Contact offset (in meters) of this entity in the physics scene.\n\nUpdating this component will update the entity's contact offset for each attached shape in the physics scene.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Contact offset"] , Description ["Contact offset (in meters) of this entity in the physics scene.\nUpdating this component will update the entity's contact offset for each attached shape in the physics scene."]] contact_offset : f32 , # [doc = "**Density**: The density of this entity.\n\nThis is used to update the `mass` when the entity is rescaled.\n\n*Attributes*: Debuggable, Networked, Store\n\n*Suggested Default*: 1.0"] @ [Debuggable , Networked , Store , Name ["Density"] , Description ["The density of this entity.\nThis is used to update the `mass` when the entity is rescaled."]] density : f32 , # [doc = "**Dynamic**: If this is true, the entity will be dynamic (i.e. be able to move). Otherwise, it will be static.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Dynamic"] , Description ["If this is true, the entity will be dynamic (i.e. be able to move). Otherwise, it will be static."]] dynamic : bool , # [doc = "**Kinematic**: If attached, and this entity is dynamic, this entity will also be kinematic (i.e. unable to be affected by other entities motion). Otherwise, it will receive forces normally.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Kinematic"] , Description ["If attached, and this entity is dynamic, this entity will also be kinematic (i.e. unable to be affected by other entities motion). Otherwise, it will receive forces normally."]] kinematic : () , # [doc = "**Linear velocity**: Linear velocity (meters/second) of this entity in the physics scene.\n\nUpdating this component will update the entity's linear velocity in the physics scene.\n\n\n\nNote that changing this component will forcibly set the velocity; changing the velocity every frame may lead to unexpected behavior, like gravity not working or collisions failing.\n\n\n\nIf you need to adjust the velocity each frame, consider applying a force using `physics` functions instead.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Linear velocity"] , Description ["Linear velocity (meters/second) of this entity in the physics scene.\nUpdating this component will update the entity's linear velocity in the physics scene.\n\nNote that changing this component will forcibly set the velocity; changing the velocity every frame may lead to unexpected behavior, like gravity not working or collisions failing.\n\nIf you need to adjust the velocity each frame, consider applying a force using `physics` functions instead."]] linear_velocity : Vec3 , # [doc = "**Make physics static**: All physics objects will be made static when loaded.\n\n*Attributes*: Debuggable, Networked, Resource, Store"] @ [Debuggable , Networked , Resource , Store , Name ["Make physics static"] , Description ["All physics objects will be made static when loaded."]] make_physics_static : bool , # [doc = "**Mass**: The mass of this entity, measured in kilograms.\n\n*Attributes*: Debuggable, Networked, Store\n\n*Suggested Default*: 1.0"] @ [Debuggable , Networked , Store , Name ["Mass"] , Description ["The mass of this entity, measured in kilograms."]] mass : f32 , # [doc = "**Physics controlled**: If attached, this entity will be controlled by physics.\n\nNote that this requires the entity to have a collider.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Physics controlled"] , Description ["If attached, this entity will be controlled by physics.\nNote that this requires the entity to have a collider."]] physics_controlled : () , # [doc = "**Plane collider**: If attached, this entity will have a plane physics collider. A plane is an infinite, flat surface. If you need a bounded flat surface, consider using a cube collider instead.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Plane collider"] , Description ["If attached, this entity will have a plane physics collider. A plane is an infinite, flat surface. If you need a bounded flat surface, consider using a cube collider instead."]] plane_collider : () , # [doc = "**Rest offset**: Rest offset (in meters) of this entity in the physics scene.\n\nUpdating this component will update the entity's rest offset for each attached shape in the physics scene.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Rest offset"] , Description ["Rest offset (in meters) of this entity in the physics scene.\nUpdating this component will update the entity's rest offset for each attached shape in the physics scene."]] rest_offset : f32 , # [doc = "**Sphere collider**: If attached, this entity will have a sphere physics collider.\n\nThe value corresponds to the radius of the sphere.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Sphere collider"] , Description ["If attached, this entity will have a sphere physics collider.\nThe value corresponds to the radius of the sphere."]] sphere_collider : f32 , # [doc = "**Unit mass**: The mass of a character/unit.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Unit mass"] , Description ["The mass of a character/unit."]] unit_mass : f32 , # [doc = "**Unit velocity**: The velocity of a character/unit.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Unit velocity"] , Description ["The velocity of a character/unit."]] unit_velocity : Vec3 , # [doc = "**Unit yaw**: The yaw of a character/unit.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Unit yaw"] , Description ["The yaw of a character/unit."]] unit_yaw : f32 , # [doc = "**Visualize collider**: If attached, the collider will be rendered.\n\n\n\n**Note**: this will continuously overwrite the `local_gizmos` component.\n\n\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Visualize collider"] , Description ["If attached, the collider will be rendered.\n\n**Note**: this will continuously overwrite the `local_gizmos` component.\n"]] visualize_collider : () , });
            }
        }
        pub mod player {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("player" , { # [doc = "**Local user ID**: The user ID of the local player.\n\n*Attributes*: Debuggable, Networked, Resource, Store"] @ [Debuggable , Networked , Resource , Store , Name ["Local user ID"] , Description ["The user ID of the local player."]] local_user_id : String , # [doc = "**Is player**: This entity is a player.\n\nNote that this is a logical construct; a player's body may be separate from the player itself.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Is player"] , Description ["This entity is a player.\nNote that this is a logical construct; a player's body may be separate from the player itself."]] is_player : () , # [doc = "**User ID**: An identifier attached to all things owned by a user, and supplied by the user.\n\nThis can be attached to more than just the player; by convention, it is also attached to related entities, including their camera and body.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["User ID"] , Description ["An identifier attached to all things owned by a user, and supplied by the user.\nThis can be attached to more than just the player; by convention, it is also attached to related entities, including their camera and body."]] user_id : String , });
            }
        }
        pub mod prefab {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("prefab" , { # [doc = "**Prefab from URL**: Load and attach a prefab from a URL or relative path.\n\nWhen loaded, the components from this prefab will add to or replace the existing components for the entity.\n\n*Attributes*: Debuggable, Store"] @ [Debuggable , Store , Name ["Prefab from URL"] , Description ["Load and attach a prefab from a URL or relative path.\nWhen loaded, the components from this prefab will add to or replace the existing components for the entity."]] prefab_from_url : String , # [doc = "**Spawned**: If attached, this entity was built from a prefab that has finished spawning.\n\n*Attributes*: Debuggable"] @ [Debuggable , Name ["Spawned"] , Description ["If attached, this entity was built from a prefab that has finished spawning."]] spawned : () , });
            }
        }
        pub mod primitives {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("primitives" , { # [doc = "**Cube**: If attached to an entity, the entity will be converted to a cube primitive.\n\nThe cube is unit-sized (i.e. 0.5 metres out to each side).\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Cube"] , Description ["If attached to an entity, the entity will be converted to a cube primitive.\nThe cube is unit-sized (i.e. 0.5 metres out to each side)."]] cube : () , # [doc = "**Quad**: If attached to an entity, the entity will be converted to a quad primitive.\n\nThe quad is unit-sized on the XY axes, and flat on the Z axis (i.e. 0.5 metres out to the XY axes).\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Quad"] , Description ["If attached to an entity, the entity will be converted to a quad primitive.\nThe quad is unit-sized on the XY axes, and flat on the Z axis (i.e. 0.5 metres out to the XY axes)."]] quad : () , # [doc = "**Sphere**: If attached to an entity alongside the other `sphere_*` components, the entity will be converted to a sphere primitive.\n\nTo easily instantiate a unit-diameter `sphere`, consider using the `Sphere` concept.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Sphere"] , Description ["If attached to an entity alongside the other `sphere_*` components, the entity will be converted to a sphere primitive.\nTo easily instantiate a unit-diameter `sphere`, consider using the `Sphere` concept."]] sphere : () , # [doc = "**Sphere radius**: Set the radius of a `sphere` entity.\n\n*Attributes*: Debuggable, Networked, Store\n\n*Suggested Default*: 0.5"] @ [Debuggable , Networked , Store , Name ["Sphere radius"] , Description ["Set the radius of a `sphere` entity."]] sphere_radius : f32 , # [doc = "**Sphere sectors**: Set the longitudinal sectors of a `sphere` entity.\n\n*Attributes*: Debuggable, Networked, Store\n\n*Suggested Default*: 36"] @ [Debuggable , Networked , Store , Name ["Sphere sectors"] , Description ["Set the longitudinal sectors of a `sphere` entity."]] sphere_sectors : u32 , # [doc = "**Sphere stacks**: Set the latitudinal stacks of a `sphere` entity.\n\n*Attributes*: Debuggable, Networked, Store\n\n*Suggested Default*: 18"] @ [Debuggable , Networked , Store , Name ["Sphere stacks"] , Description ["Set the latitudinal stacks of a `sphere` entity."]] sphere_stacks : u32 , # [doc = "**Torus**: If attached to an entity alongside the other `torus_*` components, the entity will be converted to a torus primitive.\n\nTo easily instantiate a default `torus`, consider using the `Torus` concept.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Torus"] , Description ["If attached to an entity alongside the other `torus_*` components, the entity will be converted to a torus primitive.\nTo easily instantiate a default `torus`, consider using the `Torus` concept."]] torus : () , # [doc = "**Torus inner radius**: Set the inner radius of a `torus` entity, spanning XY-plane.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Torus inner radius"] , Description ["Set the inner radius of a `torus` entity, spanning XY-plane."]] torus_inner_radius : f32 , # [doc = "**Torus outer radius**: Set the outer radius of a `torus` entity, spanning XY-plane.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Torus outer radius"] , Description ["Set the outer radius of a `torus` entity, spanning XY-plane."]] torus_outer_radius : f32 , # [doc = "**Torus loops**: Set the loops of a `torus` entity, spanning XY-plane.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Torus loops"] , Description ["Set the loops of a `torus` entity, spanning XY-plane."]] torus_loops : u32 , # [doc = "**Torus slices**: Set the slices of a `torus` entity, spanning XY-plane.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Torus slices"] , Description ["Set the slices of a `torus` entity, spanning XY-plane."]] torus_slices : u32 , # [doc = "**Capsule**: If attached to an entity alongside the other `capsule_*` components, the entity will be converted to a capsule primitive.\n\nTo easily instantiate a default `capsule`, consider using the `Capsule` concept.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Capsule"] , Description ["If attached to an entity alongside the other `capsule_*` components, the entity will be converted to a capsule primitive.\nTo easily instantiate a default `capsule`, consider using the `Capsule` concept."]] capsule : () , # [doc = "**Capsule radius**: Set the radius of a `capsule` entity, spanning XY-plane.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Capsule radius"] , Description ["Set the radius of a `capsule` entity, spanning XY-plane."]] capsule_radius : f32 , # [doc = "**Capsule half-height**: Set the half-height of the `capsule` entity, spanning Z-axis, excluding the caps.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Capsule half-height"] , Description ["Set the half-height of the `capsule` entity, spanning Z-axis, excluding the caps."]] capsule_half_height : f32 , # [doc = "**Capsule rings**: Set the number of sections between the caps.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Capsule rings"] , Description ["Set the number of sections between the caps."]] capsule_rings : u32 , # [doc = "**Capsule latitudes**: Set the number of latitudinal sections. Should be even.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Capsule latitudes"] , Description ["Set the number of latitudinal sections. Should be even."]] capsule_latitudes : u32 , # [doc = "**Capsule longitudes**: Set the number of longitudinal sections.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Capsule longitudes"] , Description ["Set the number of longitudinal sections."]] capsule_longitudes : u32 , });
            }
        }
        pub mod procedurals {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("procedurals" , { # [doc = "**Procedural mesh**: Attaches a procedural mesh to this entity\n\n*Attributes*: Debuggable, Store"] @ [Debuggable , Store , Name ["Procedural mesh"] , Description ["Attaches a procedural mesh to this entity"]] procedural_mesh : ProceduralMeshHandle , # [doc = "**Procedural material**: Attaches a procedural material to this entity\n\n*Attributes*: Debuggable, Store"] @ [Debuggable , Store , Name ["Procedural material"] , Description ["Attaches a procedural material to this entity"]] procedural_material : ProceduralMaterialHandle , });
            }
        }
        pub mod rect {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("rect" , { # [doc = "**Background color**: Background color of an entity with a `rect` component.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Background color"] , Description ["Background color of an entity with a `rect` component."]] background_color : Vec4 , # [doc = "**Background URL**: URL to an image asset.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Background URL"] , Description ["URL to an image asset."]] background_url : String , # [doc = "**Border color**: Border color of an entity with a `rect` component.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Border color"] , Description ["Border color of an entity with a `rect` component."]] border_color : Vec4 , # [doc = "**Border radius**: Radius for each corner of an entity with a `rect` component.\n\n`x` = top-left, `y` = top-right, `z` = bottom-left, `w` = bottom-right.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Border radius"] , Description ["Radius for each corner of an entity with a `rect` component.\n`x` = top-left, `y` = top-right, `z` = bottom-left, `w` = bottom-right."]] border_radius : Vec4 , # [doc = "**Border thickness**: Border thickness of an entity with a `rect` component.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Border thickness"] , Description ["Border thickness of an entity with a `rect` component."]] border_thickness : f32 , # [doc = "**Pixel Line from**: Start point of a pixel sized line.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Pixel Line from"] , Description ["Start point of a pixel sized line."]] pixel_line_from : Vec3 , # [doc = "**Pixel Line to**: End point of a pixel sized line.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Pixel Line to"] , Description ["End point of a pixel sized line."]] pixel_line_to : Vec3 , # [doc = "**Line from**: Start point of a line.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Line from"] , Description ["Start point of a line."]] line_from : Vec3 , # [doc = "**Line to**: End point of a line.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Line to"] , Description ["End point of a line."]] line_to : Vec3 , # [doc = "**Line width**: Width of line.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Line width"] , Description ["Width of line."]] line_width : f32 , # [doc = "**Rect**: If attached to an entity, the entity will be converted to a UI rectangle, with optionally rounded corners and borders.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Rect"] , Description ["If attached to an entity, the entity will be converted to a UI rectangle, with optionally rounded corners and borders."]] rect : () , # [doc = "**Size from background image**: Resize this rect based on the size of the background image.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Size from background image"] , Description ["Resize this rect based on the size of the background image."]] size_from_background_image : () , });
            }
        }
        pub mod rendering {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("rendering" , { # [doc = "**Cast shadows**: If attached, this entity will cast shadows.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Cast shadows"] , Description ["If attached, this entity will cast shadows."]] cast_shadows : () , # [doc = "**Color**: This entity will be tinted with the specified color if the color is not black.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Color"] , Description ["This entity will be tinted with the specified color if the color is not black."]] color : Vec4 , # [doc = "**Double-sided**: If attached, this controls whether or not the entity will be rendered with double-sided rendering. If not attached, the decision will fall back to the material.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Double-sided"] , Description ["If attached, this controls whether or not the entity will be rendered with double-sided rendering. If not attached, the decision will fall back to the material."]] double_sided : bool , # [doc = "**Fog color**: The color of the fog for this `sun`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Fog color"] , Description ["The color of the fog for this `sun`."]] fog_color : Vec3 , # [doc = "**Fog density**: The density of the fog for this `sun`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Fog density"] , Description ["The density of the fog for this `sun`."]] fog_density : f32 , # [doc = "**Fog height fall-off**: The height at which the fog will fall off (i.e. stop being visible) for this `sun`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Fog height fall-off"] , Description ["The height at which the fog will fall off (i.e. stop being visible) for this `sun`."]] fog_height_falloff : f32 , # [doc = "**Joint Matrices**: Contains the matrices for each joint of this skinned mesh.\n\nThis should be used in combination with `joints`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Joint Matrices"] , Description ["Contains the matrices for each joint of this skinned mesh.\nThis should be used in combination with `joints`."]] joint_matrices : Vec :: < Mat4 > , # [doc = "**Joints**: Contains the joints that comprise this skinned mesh.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Joints"] , Description ["Contains the joints that comprise this skinned mesh."]] joints : Vec :: < EntityId > , # [doc = "**Light ambient**: The ambient light color of the `sun`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Light ambient"] , Description ["The ambient light color of the `sun`."]] light_ambient : Vec3 , # [doc = "**Light diffuse**: The diffuse light color of the `sun`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Light diffuse"] , Description ["The diffuse light color of the `sun`."]] light_diffuse : Vec3 , # [doc = "**Outline**: If attached, this entity will be rendered with an outline with the color specified.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Outline"] , Description ["If attached, this entity will be rendered with an outline with the color specified."]] outline : Vec4 , # [doc = "**Outline (recursive)**: If attached, this entity and all of its children will be rendered with an outline with the color specified.\n\nYou do not need to attach `outline` if you have attached `outline_recursive`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Outline (recursive)"] , Description ["If attached, this entity and all of its children will be rendered with an outline with the color specified.\nYou do not need to attach `outline` if you have attached `outline_recursive`."]] outline_recursive : Vec4 , # [doc = "**Overlay**: If attached, this entity will be rendered with an overlay.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Overlay"] , Description ["If attached, this entity will be rendered with an overlay."]] overlay : () , # [doc = "**PBR material from URL**: Load a PBR material from the URL and attach it to this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["PBR material from URL"] , Description ["Load a PBR material from the URL and attach it to this entity."]] pbr_material_from_url : String , # [doc = "**Sky**: Add a realistic skybox to the scene.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Sky"] , Description ["Add a realistic skybox to the scene."]] sky : () , # [doc = "**Sun**: Marks this entity as a sun (i.e. its rotation will be used to control the global light direction).\n\nThe entity with the highest `sun` value takes precedence.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Sun"] , Description ["Marks this entity as a sun (i.e. its rotation will be used to control the global light direction).\nThe entity with the highest `sun` value takes precedence."]] sun : f32 , # [doc = "**Transparency group**: Controls when this transparent object will be rendered. Transparent objects are sorted by `(transparency_group, z-depth)`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Transparency group"] , Description ["Controls when this transparent object will be rendered. Transparent objects are sorted by `(transparency_group, z-depth)`."]] transparency_group : i32 , # [doc = "**Water**: Add a realistic water plane to this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Water"] , Description ["Add a realistic water plane to this entity."]] water : () , # [doc = "**Decal material from URL**: Load a Decal material from the URL and attach it to this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Decal material from URL"] , Description ["Load a Decal material from the URL and attach it to this entity."]] decal_from_url : String , # [doc = "**Scissors**: Apply a scissors test to this entity (anything outside the rect will be hidden).\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Scissors"] , Description ["Apply a scissors test to this entity (anything outside the rect will be hidden)."]] scissors : UVec4 , # [doc = "**Scissors (recursive)**: If attached, this entity and all of its children will be rendered with an scissor with the rect specified.\n\nYou do not need to attach `scissors` if you have attached `scissors_recursive`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Scissors (recursive)"] , Description ["If attached, this entity and all of its children will be rendered with an scissor with the rect specified.\nYou do not need to attach `scissors` if you have attached `scissors_recursive`."]] scissors_recursive : UVec4 , # [doc = "**Local bounding AABB min**: The minimum point of the local AABB of this entity.\n\n*Attributes*: Debuggable"] @ [Debuggable , Name ["Local bounding AABB min"] , Description ["The minimum point of the local AABB of this entity."]] local_bounding_aabb_min : Vec3 , # [doc = "**Local bounding AABB max**: The maximum point of the local AABB of this entity.\n\n*Attributes*: Debuggable"] @ [Debuggable , Name ["Local bounding AABB max"] , Description ["The maximum point of the local AABB of this entity."]] local_bounding_aabb_max : Vec3 , # [doc = "**World bounding AABB min**: The minimum point of the world AABB of this entity.\n\n*Attributes*: Debuggable"] @ [Debuggable , Name ["World bounding AABB min"] , Description ["The minimum point of the world AABB of this entity."]] world_bounding_aabb_min : Vec3 , # [doc = "**World bounding AABB max**: The maximum point of the world AABB of this entity.\n\n*Attributes*: Debuggable"] @ [Debuggable , Name ["World bounding AABB max"] , Description ["The maximum point of the world AABB of this entity."]] world_bounding_aabb_max : Vec3 , # [doc = "**World bounding sphere center**: The center of the world bounding sphere of this entity.\n\n*Attributes*: Debuggable"] @ [Debuggable , Name ["World bounding sphere center"] , Description ["The center of the world bounding sphere of this entity."]] world_bounding_sphere_center : Vec3 , # [doc = "**World bounding sphere radius**: The radius of the world bounding sphere of this entity.\n\n*Attributes*: Debuggable"] @ [Debuggable , Name ["World bounding sphere radius"] , Description ["The radius of the world bounding sphere of this entity."]] world_bounding_sphere_radius : f32 , });
            }
        }
        pub mod text {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("text" , { # [doc = "**Font family**: Font family to be used. Can either be 'Default', 'FontAwesome', 'FontAwesomeSolid', 'Code' or a url to a font.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Font family"] , Description ["Font family to be used. Can either be 'Default', 'FontAwesome', 'FontAwesomeSolid', 'Code' or a url to a font."]] font_family : String , # [doc = "**Font size**: Size of the font.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Font size"] , Description ["Size of the font."]] font_size : f32 , # [doc = "**Font style**: Style of the font.\n\n*Attributes*: Debuggable, Networked, Store, Enum"] @ [Debuggable , Networked , Store , Enum , Name ["Font style"] , Description ["Style of the font."]] font_style : crate :: generated :: raw :: ambient_core :: text :: types :: FontStyle , # [doc = "**Text**: Create a text mesh on this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Text"] , Description ["Create a text mesh on this entity."]] text : String , });
            }
            #[doc = r" Auto-generated type definitions."]
            pub mod types {
                use ambient_package_rt::message_serde::*;
                use serde;
                #[derive(
                    Copy,
                    Clone,
                    Debug,
                    PartialEq,
                    Eq,
                    serde :: Serialize,
                    serde :: Deserialize,
                    Default,
                )]
                #[serde(crate = "self::serde")]
                #[doc = "**FontStyle**: Style of the font."]
                pub enum FontStyle {
                    #[default]
                    #[doc = "Use bold for this text."]
                    Bold,
                    #[doc = "Use bold italic for this text."]
                    BoldItalic,
                    #[doc = "Use medium for this text."]
                    Medium,
                    #[doc = "Use medium italic for this text."]
                    MediumItalic,
                    #[doc = "Use regular for this text."]
                    Regular,
                    #[doc = "Use italic for this text."]
                    Italic,
                    #[doc = "Use light for this text."]
                    Light,
                    #[doc = "Use light italic for this text."]
                    LightItalic,
                }
                impl crate::EnumComponent for FontStyle {
                    fn to_u32(&self) -> u32 {
                        match self {
                            Self::Bold => FontStyle::Bold as u32,
                            Self::BoldItalic => FontStyle::BoldItalic as u32,
                            Self::Medium => FontStyle::Medium as u32,
                            Self::MediumItalic => FontStyle::MediumItalic as u32,
                            Self::Regular => FontStyle::Regular as u32,
                            Self::Italic => FontStyle::Italic as u32,
                            Self::Light => FontStyle::Light as u32,
                            Self::LightItalic => FontStyle::LightItalic as u32,
                        }
                    }
                    fn from_u32(value: u32) -> Option<Self> {
                        if value == FontStyle::Bold as u32 {
                            return Some(Self::Bold);
                        }
                        if value == FontStyle::BoldItalic as u32 {
                            return Some(Self::BoldItalic);
                        }
                        if value == FontStyle::Medium as u32 {
                            return Some(Self::Medium);
                        }
                        if value == FontStyle::MediumItalic as u32 {
                            return Some(Self::MediumItalic);
                        }
                        if value == FontStyle::Regular as u32 {
                            return Some(Self::Regular);
                        }
                        if value == FontStyle::Italic as u32 {
                            return Some(Self::Italic);
                        }
                        if value == FontStyle::Light as u32 {
                            return Some(Self::Light);
                        }
                        if value == FontStyle::LightItalic as u32 {
                            return Some(Self::LightItalic);
                        }
                        None
                    }
                }
                impl MessageSerde for FontStyle {
                    fn serialize_message_part(
                        &self,
                        output: &mut Vec<u8>,
                    ) -> Result<(), MessageSerdeError> {
                        crate::EnumComponent::to_u32(self).serialize_message_part(output)
                    }
                    fn deserialize_message_part(
                        input: &mut dyn std::io::Read,
                    ) -> Result<Self, MessageSerdeError> {
                        crate::EnumComponent::from_u32(u32::deserialize_message_part(input)?)
                            .ok_or(MessageSerdeError::InvalidValue)
                    }
                }
            }
        }
        pub mod transform {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("transform" , { # [doc = "**Cylindrical billboard Z**: If attached, this ensures this entity is always aligned with the camera, except on the Z-axis.\n\nThis is useful for decorations that the player will be looking at from roughly the same altitude.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Cylindrical billboard Z"] , Description ["If attached, this ensures this entity is always aligned with the camera, except on the Z-axis.\nThis is useful for decorations that the player will be looking at from roughly the same altitude."]] cylindrical_billboard_z : () , # [doc = "**Euler rotation**: The Euler rotation of this entity in ZYX order.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Euler rotation"] , Description ["The Euler rotation of this entity in ZYX order."]] euler_rotation : Vec3 , # [doc = "**Inverse Local to World**: Converts a world position to a local position.\n\nThis is automatically updated.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Inverse Local to World"] , Description ["Converts a world position to a local position.\nThis is automatically updated."]] inv_local_to_world : Mat4 , # [doc = "**Local to Parent**: Transformation from the entity's local space to the parent's space.\n\n*Attributes*: Debuggable, Networked, Store, MaybeResource"] @ [Debuggable , Networked , Store , MaybeResource , Name ["Local to Parent"] , Description ["Transformation from the entity's local space to the parent's space."]] local_to_parent : Mat4 , # [doc = "**Local to World**: Transformation from the entity's local space to worldspace.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Local to World"] , Description ["Transformation from the entity's local space to worldspace."]] local_to_world : Mat4 , # [doc = "**Look-at target**: The position that this entity should be looking at.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Look-at target"] , Description ["The position that this entity should be looking at."]] lookat_target : Vec3 , # [doc = "**Look-at up**: When combined with `lookat_target`, the up vector for this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Look-at up"] , Description ["When combined with `lookat_target`, the up vector for this entity."]] lookat_up : Vec3 , # [doc = "**Mesh to Local**: Transformation from mesh-space to the entity's local space.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Mesh to Local"] , Description ["Transformation from mesh-space to the entity's local space."]] mesh_to_local : Mat4 , # [doc = "**Mesh to World**: Transformation from mesh-space to world space.\n\nThis is automatically updated when `mesh_to_local` and `local_to_world` change.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Mesh to World"] , Description ["Transformation from mesh-space to world space.\nThis is automatically updated when `mesh_to_local` and `local_to_world` change."]] mesh_to_world : Mat4 , # [doc = "**Reset scale**: If attached to a transform hierarchy, the scale will be reset at that point, with only rotation/translation considered.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Reset scale"] , Description ["If attached to a transform hierarchy, the scale will be reset at that point, with only rotation/translation considered."]] reset_scale : () , # [doc = "**Rotation**: The rotation of this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Rotation"] , Description ["The rotation of this entity."]] rotation : Quat , # [doc = "**Scale**: The scale of this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Scale"] , Description ["The scale of this entity."]] scale : Vec3 , # [doc = "**Spherical billboard**: If attached, this ensures that this entity is always aligned with the camera.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Spherical billboard"] , Description ["If attached, this ensures that this entity is always aligned with the camera."]] spherical_billboard : () , # [doc = "**Translation**: The translation/position of this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Translation"] , Description ["The translation/position of this entity."]] translation : Vec3 , });
            }
        }
        pub mod ui {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("ui" , { # [doc = "**Focus**: Currently focused object.\n\n*Attributes*: Debuggable, Networked, Resource"] @ [Debuggable , Networked , Resource , Name ["Focus"] , Description ["Currently focused object."]] focus : String , # [doc = "**Focus**: This entity can be focused. The value is the focus id.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Focus"] , Description ["This entity can be focused. The value is the focus id."]] focusable : String , });
            }
            #[doc = r" Auto-generated message definitions. Messages are used to communicate with the runtime, the other side of the network,"]
            #[doc = r" and with other modules."]
            pub mod messages {
                use crate::{Entity, EntityId};
                use ambient_package_rt::message_serde::{
                    Message, MessageSerde, MessageSerdeError, ModuleMessage, RuntimeMessage,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                #[derive(Clone, Debug)]
                #[doc = "**FocusChanged**: Focus has been updated"]
                pub struct FocusChanged {
                    pub from_external: bool,
                    pub focus: String,
                }
                impl FocusChanged {
                    #[allow(clippy::too_many_arguments)]
                    pub fn new(from_external: impl Into<bool>, focus: impl Into<String>) -> Self {
                        Self {
                            from_external: from_external.into(),
                            focus: focus.into(),
                        }
                    }
                }
                impl Message for FocusChanged {
                    fn id() -> &'static str {
                        "ambient_core::ui::FocusChanged"
                    }
                    fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                        let mut output = vec![];
                        self.from_external.serialize_message_part(&mut output)?;
                        self.focus.serialize_message_part(&mut output)?;
                        Ok(output)
                    }
                    fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                        Ok(Self {
                            from_external: bool::deserialize_message_part(&mut input)?,
                            focus: String::deserialize_message_part(&mut input)?,
                        })
                    }
                }
                impl ModuleMessage for FocusChanged {}
            }
        }
        pub mod wasm {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    components, Debuggable, Description, EntityId, Enum, MaybeResource, Name,
                    Networked, Resource, Store,
                };
                use ambient_shared_types::{
                    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
                    ProceduralTextureHandle,
                };
                use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
                use std::time::Duration;
                components ! ("wasm" , { # [doc = "**Is module**: A module.\n\n*Attributes*: Networked, Store, Debuggable"] @ [Networked , Store , Debuggable , Name ["Is module"] , Description ["A module."]] is_module : () , # [doc = "**Is module on server**: Whether or not this module is on the server.\n\n*Attributes*: Networked, Store, Debuggable"] @ [Networked , Store , Debuggable , Name ["Is module on server"] , Description ["Whether or not this module is on the server."]] is_module_on_server : () , # [doc = "**Bytecode from URL**: Asset URL for the bytecode of a WASM component.\n\n*Attributes*: Networked, Store, Debuggable"] @ [Networked , Store , Debuggable , Name ["Bytecode from URL"] , Description ["Asset URL for the bytecode of a WASM component."]] bytecode_from_url : String , # [doc = "**Module enabled**: Whether or not this module is enabled.\n\n*Attributes*: Networked, Store, Debuggable"] @ [Networked , Store , Debuggable , Name ["Module enabled"] , Description ["Whether or not this module is enabled."]] module_enabled : bool , # [doc = "**Module name**: The name of this module.\n\n*Attributes*: Networked, Store, Debuggable"] @ [Networked , Store , Debuggable , Name ["Module name"] , Description ["The name of this module."]] module_name : String , # [doc = "**Package reference**: The package that this module belongs to.\n\n*Attributes*: Networked, Store, Debuggable"] @ [Networked , Store , Debuggable , Name ["Package reference"] , Description ["The package that this module belongs to."]] package_ref : EntityId , });
            }
        }
        #[doc = r" Auto-generated message definitions. Messages are used to communicate with the runtime, the other side of the network,"]
        #[doc = r" and with other modules."]
        pub mod messages {
            use crate::{Entity, EntityId};
            use ambient_package_rt::message_serde::{
                Message, MessageSerde, MessageSerdeError, ModuleMessage, RuntimeMessage,
            };
            use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
            #[derive(Clone, Debug)]
            #[doc = "**Frame**: Sent to all modules every frame."]
            pub struct Frame;
            impl Frame {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for Frame {
                fn id() -> &'static str {
                    "ambient_core::Frame"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl RuntimeMessage for Frame {}
            impl Default for Frame {
                fn default() -> Self {
                    Self::new()
                }
            }
            #[derive(Clone, Debug)]
            #[doc = "**Collision**: Sent when a collision occurs. The points and normals are in world space."]
            pub struct Collision {
                pub ids: Vec<EntityId>,
                pub points: Vec<Vec3>,
                pub normals: Vec<Vec3>,
            }
            impl Collision {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    ids: impl Into<Vec<EntityId>>,
                    points: impl Into<Vec<Vec3>>,
                    normals: impl Into<Vec<Vec3>>,
                ) -> Self {
                    Self {
                        ids: ids.into(),
                        points: points.into(),
                        normals: normals.into(),
                    }
                }
            }
            impl Message for Collision {
                fn id() -> &'static str {
                    "ambient_core::Collision"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.ids.serialize_message_part(&mut output)?;
                    self.points.serialize_message_part(&mut output)?;
                    self.normals.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        ids: Vec::<EntityId>::deserialize_message_part(&mut input)?,
                        points: Vec::<Vec3>::deserialize_message_part(&mut input)?,
                        normals: Vec::<Vec3>::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for Collision {}
            #[derive(Clone, Debug)]
            #[doc = "**ColliderLoads**: Sent when colliders load."]
            pub struct ColliderLoads {
                pub ids: Vec<EntityId>,
            }
            impl ColliderLoads {
                #[allow(clippy::too_many_arguments)]
                pub fn new(ids: impl Into<Vec<EntityId>>) -> Self {
                    Self { ids: ids.into() }
                }
            }
            impl Message for ColliderLoads {
                fn id() -> &'static str {
                    "ambient_core::ColliderLoads"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.ids.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        ids: Vec::<EntityId>::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for ColliderLoads {}
            #[derive(Clone, Debug)]
            #[doc = "**ModuleLoad**: Sent to a module when it loads."]
            pub struct ModuleLoad;
            impl ModuleLoad {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for ModuleLoad {
                fn id() -> &'static str {
                    "ambient_core::ModuleLoad"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl RuntimeMessage for ModuleLoad {}
            impl Default for ModuleLoad {
                fn default() -> Self {
                    Self::new()
                }
            }
            #[derive(Clone, Debug)]
            #[doc = "**ModuleUnload**: Sent to a module when it unloads."]
            pub struct ModuleUnload;
            impl ModuleUnload {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for ModuleUnload {
                fn id() -> &'static str {
                    "ambient_core::ModuleUnload"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl RuntimeMessage for ModuleUnload {}
            impl Default for ModuleUnload {
                fn default() -> Self {
                    Self::new()
                }
            }
            #[derive(Clone, Debug)]
            #[doc = "**WindowFocusChange**: Sent when the window gains or loses focus."]
            pub struct WindowFocusChange {
                pub focused: bool,
            }
            impl WindowFocusChange {
                #[allow(clippy::too_many_arguments)]
                pub fn new(focused: impl Into<bool>) -> Self {
                    Self {
                        focused: focused.into(),
                    }
                }
            }
            impl Message for WindowFocusChange {
                fn id() -> &'static str {
                    "ambient_core::WindowFocusChange"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.focused.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        focused: bool::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for WindowFocusChange {}
            #[derive(Clone, Debug)]
            #[doc = "**WindowClose**: Sent when the window is closed."]
            pub struct WindowClose;
            impl WindowClose {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for WindowClose {
                fn id() -> &'static str {
                    "ambient_core::WindowClose"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl RuntimeMessage for WindowClose {}
            impl Default for WindowClose {
                fn default() -> Self {
                    Self::new()
                }
            }
            #[derive(Clone, Debug)]
            #[doc = "**WindowKeyboardCharacter**: Sent when the window receives a character from the keyboard."]
            pub struct WindowKeyboardCharacter {
                pub character: String,
            }
            impl WindowKeyboardCharacter {
                #[allow(clippy::too_many_arguments)]
                pub fn new(character: impl Into<String>) -> Self {
                    Self {
                        character: character.into(),
                    }
                }
            }
            impl Message for WindowKeyboardCharacter {
                fn id() -> &'static str {
                    "ambient_core::WindowKeyboardCharacter"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.character.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        character: String::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for WindowKeyboardCharacter {}
            #[derive(Clone, Debug)]
            #[doc = "**WindowKeyboardModifiersChange**: Sent when the window's keyboard modifiers change."]
            pub struct WindowKeyboardModifiersChange {
                pub modifiers: u32,
            }
            impl WindowKeyboardModifiersChange {
                #[allow(clippy::too_many_arguments)]
                pub fn new(modifiers: impl Into<u32>) -> Self {
                    Self {
                        modifiers: modifiers.into(),
                    }
                }
            }
            impl Message for WindowKeyboardModifiersChange {
                fn id() -> &'static str {
                    "ambient_core::WindowKeyboardModifiersChange"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.modifiers.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        modifiers: u32::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for WindowKeyboardModifiersChange {}
            #[derive(Clone, Debug)]
            #[doc = "**WindowKeyboardInput**: Sent when the window receives a keyboard input."]
            pub struct WindowKeyboardInput {
                pub pressed: bool,
                pub modifiers: u32,
                pub keycode: Option<String>,
            }
            impl WindowKeyboardInput {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    pressed: impl Into<bool>,
                    modifiers: impl Into<u32>,
                    keycode: impl Into<Option<String>>,
                ) -> Self {
                    Self {
                        pressed: pressed.into(),
                        modifiers: modifiers.into(),
                        keycode: keycode.into(),
                    }
                }
            }
            impl Message for WindowKeyboardInput {
                fn id() -> &'static str {
                    "ambient_core::WindowKeyboardInput"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.pressed.serialize_message_part(&mut output)?;
                    self.modifiers.serialize_message_part(&mut output)?;
                    self.keycode.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        pressed: bool::deserialize_message_part(&mut input)?,
                        modifiers: u32::deserialize_message_part(&mut input)?,
                        keycode: Option::<String>::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for WindowKeyboardInput {}
            #[derive(Clone, Debug)]
            #[doc = "**WindowMouseInput**: Sent when the window receives a mouse input."]
            pub struct WindowMouseInput {
                pub pressed: bool,
                pub button: u32,
            }
            impl WindowMouseInput {
                #[allow(clippy::too_many_arguments)]
                pub fn new(pressed: impl Into<bool>, button: impl Into<u32>) -> Self {
                    Self {
                        pressed: pressed.into(),
                        button: button.into(),
                    }
                }
            }
            impl Message for WindowMouseInput {
                fn id() -> &'static str {
                    "ambient_core::WindowMouseInput"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.pressed.serialize_message_part(&mut output)?;
                    self.button.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        pressed: bool::deserialize_message_part(&mut input)?,
                        button: u32::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for WindowMouseInput {}
            #[derive(Clone, Debug)]
            #[doc = "**WindowMouseWheel**: Sent when the window receives a mouse wheel input."]
            pub struct WindowMouseWheel {
                pub delta: Vec2,
                pub pixels: bool,
            }
            impl WindowMouseWheel {
                #[allow(clippy::too_many_arguments)]
                pub fn new(delta: impl Into<Vec2>, pixels: impl Into<bool>) -> Self {
                    Self {
                        delta: delta.into(),
                        pixels: pixels.into(),
                    }
                }
            }
            impl Message for WindowMouseWheel {
                fn id() -> &'static str {
                    "ambient_core::WindowMouseWheel"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.delta.serialize_message_part(&mut output)?;
                    self.pixels.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        delta: Vec2::deserialize_message_part(&mut input)?,
                        pixels: bool::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for WindowMouseWheel {}
            #[derive(Clone, Debug)]
            #[doc = "**WindowMouseMotion**: Sent when the window receives a mouse motion input."]
            pub struct WindowMouseMotion {
                pub delta: Vec2,
            }
            impl WindowMouseMotion {
                #[allow(clippy::too_many_arguments)]
                pub fn new(delta: impl Into<Vec2>) -> Self {
                    Self {
                        delta: delta.into(),
                    }
                }
            }
            impl Message for WindowMouseMotion {
                fn id() -> &'static str {
                    "ambient_core::WindowMouseMotion"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.delta.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        delta: Vec2::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for WindowMouseMotion {}
            #[derive(Clone, Debug)]
            #[doc = "**WindowCursorLockChange**: Sent when the window's cursor lock changes."]
            pub struct WindowCursorLockChange {
                pub locked: bool,
            }
            impl WindowCursorLockChange {
                #[allow(clippy::too_many_arguments)]
                pub fn new(locked: impl Into<bool>) -> Self {
                    Self {
                        locked: locked.into(),
                    }
                }
            }
            impl Message for WindowCursorLockChange {
                fn id() -> &'static str {
                    "ambient_core::WindowCursorLockChange"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.locked.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        locked: bool::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for WindowCursorLockChange {}
            #[derive(Clone, Debug)]
            #[doc = "**HttpResponse**: Sent when an HTTP response is received."]
            pub struct HttpResponse {
                pub response_id: u64,
                pub status: u32,
                pub body: Vec<u8>,
                pub error: Option<String>,
            }
            impl HttpResponse {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    response_id: impl Into<u64>,
                    status: impl Into<u32>,
                    body: impl Into<Vec<u8>>,
                    error: impl Into<Option<String>>,
                ) -> Self {
                    Self {
                        response_id: response_id.into(),
                        status: status.into(),
                        body: body.into(),
                        error: error.into(),
                    }
                }
            }
            impl Message for HttpResponse {
                fn id() -> &'static str {
                    "ambient_core::HttpResponse"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.response_id.serialize_message_part(&mut output)?;
                    self.status.serialize_message_part(&mut output)?;
                    self.body.serialize_message_part(&mut output)?;
                    self.error.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        response_id: u64::deserialize_message_part(&mut input)?,
                        status: u32::deserialize_message_part(&mut input)?,
                        body: Vec::<u8>::deserialize_message_part(&mut input)?,
                        error: Option::<String>::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for HttpResponse {}
            #[derive(Clone, Debug)]
            #[doc = "**WasmRebuild**: Sent when a request for WASM rebuilding is completed."]
            pub struct WasmRebuild {
                pub error: Option<String>,
            }
            impl WasmRebuild {
                #[allow(clippy::too_many_arguments)]
                pub fn new(error: impl Into<Option<String>>) -> Self {
                    Self {
                        error: error.into(),
                    }
                }
            }
            impl Message for WasmRebuild {
                fn id() -> &'static str {
                    "ambient_core::WasmRebuild"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.error.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        error: Option::<String>::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for WasmRebuild {}
        }
        #[doc = r" Auto-generated type definitions."]
        pub mod types {
            use ambient_package_rt::message_serde::*;
            use serde;
            #[derive(
                Copy, Clone, Debug, PartialEq, Eq, serde :: Serialize, serde :: Deserialize, Default,
            )]
            #[serde(crate = "self::serde")]
            #[doc = "**HttpMethod**: The HTTP method."]
            pub enum HttpMethod {
                #[default]
                #[doc = "GET"]
                Get,
                #[doc = "POST"]
                Post,
            }
            impl crate::EnumComponent for HttpMethod {
                fn to_u32(&self) -> u32 {
                    match self {
                        Self::Get => HttpMethod::Get as u32,
                        Self::Post => HttpMethod::Post as u32,
                    }
                }
                fn from_u32(value: u32) -> Option<Self> {
                    if value == HttpMethod::Get as u32 {
                        return Some(Self::Get);
                    }
                    if value == HttpMethod::Post as u32 {
                        return Some(Self::Post);
                    }
                    None
                }
            }
            impl MessageSerde for HttpMethod {
                fn serialize_message_part(
                    &self,
                    output: &mut Vec<u8>,
                ) -> Result<(), MessageSerdeError> {
                    crate::EnumComponent::to_u32(self).serialize_message_part(output)
                }
                fn deserialize_message_part(
                    input: &mut dyn std::io::Read,
                ) -> Result<Self, MessageSerdeError> {
                    crate::EnumComponent::from_u32(u32::deserialize_message_part(input)?)
                        .ok_or(MessageSerdeError::InvalidValue)
                }
            }
        }
        pub fn init() {
            crate::generated::raw::ambient_core::animation::components::init_components();
            crate::generated::raw::ambient_core::app::components::init_components();
            crate::generated::raw::ambient_core::audio::components::init_components();
            crate::generated::raw::ambient_core::camera::components::init_components();
            crate::generated::raw::ambient_core::ecs::components::init_components();
            crate::generated::raw::ambient_core::hierarchy::components::init_components();
            crate::generated::raw::ambient_core::input::components::init_components();
            crate::generated::raw::ambient_core::layout::components::init_components();
            crate::generated::raw::ambient_core::model::components::init_components();
            crate::generated::raw::ambient_core::network::components::init_components();
            crate::generated::raw::ambient_core::package::components::init_components();
            crate::generated::raw::ambient_core::physics::components::init_components();
            crate::generated::raw::ambient_core::player::components::init_components();
            crate::generated::raw::ambient_core::prefab::components::init_components();
            crate::generated::raw::ambient_core::primitives::components::init_components();
            crate::generated::raw::ambient_core::procedurals::components::init_components();
            crate::generated::raw::ambient_core::rect::components::init_components();
            crate::generated::raw::ambient_core::rendering::components::init_components();
            crate::generated::raw::ambient_core::text::components::init_components();
            crate::generated::raw::ambient_core::transform::components::init_components();
            crate::generated::raw::ambient_core::ui::components::init_components();
            crate::generated::raw::ambient_core::wasm::components::init_components();
        }
    }
}
