use crate::{
    core::{
        animation::components::{
            apply_base_pose, bind_id, bind_ids, blend, clip_duration, freeze_at_percentage,
            freeze_at_time, is_animation_player, looping, mask_bind_ids, mask_weights,
            play_clip_from_url, retarget_animation_scaled, retarget_model_from_url, start_time,
        },
        app::components::name,
        hierarchy::components::{children, parent},
    },
    entity,
    prelude::{epoch_time, Entity, EntityId},
};
use std::time::Duration;

/// This plays animations, and can handle blending and masking of animations together to create
/// complex effects. A single animation player can be attached to multiple entities in the scene.
///
/// This is just a reference to an entity which lives in the ecs. You need to call `despawn` to
/// remove it.
#[derive(Debug, Clone, Copy)]
pub struct AnimationPlayerRef(pub EntityId);
impl AnimationPlayerRef {
    /// Create a new animation player, with `root` as the currently playing node
    pub fn new(root: impl AsRef<AnimationNodeRef>) -> Self {
        let root: &AnimationNodeRef = root.as_ref();
        let player = Entity::new()
            .with(is_animation_player(), ())
            .with(children(), vec![root.0])
            .with(name(), "Animation player".to_string())
            .spawn();
        entity::add_component(root.0, parent(), player);
        Self(player)
    }
    fn root(&self) -> Option<AnimationNodeRef> {
        if let Some(children) = entity::get_component(self.0, children()) {
            children.first().copied().map(AnimationNodeRef)
        } else {
            None
        }
    }
    /// Replaces the current root node of the animation player with a new node
    pub fn play(&self, node: impl AsRef<AnimationNodeRef>) -> Option<AnimationNodeRef> {
        let old_root = self.root();
        let new_root: &AnimationNodeRef = node.as_ref();
        entity::add_component(self.0, children(), vec![new_root.0]);
        entity::add_component(new_root.0, parent(), self.0);
        old_root
    }
    /// Despawn this animation player.
    /// Note that dropping this player won't despawn it automatically; only call this method will despawn it.
    pub fn despawn(self) {
        if let Some(root) = self.root() {
            root.despawn();
        }
        entity::despawn(self.0);
    }
}

/// An animation node. Used in the animation player.
#[derive(Debug, Clone, Copy)]
pub struct AnimationNodeRef(EntityId);
impl AnimationNodeRef {
    /// Get the entity id of this node
    pub fn get_entity_id(&self) -> EntityId {
        self.0
    }
    /// Use an existing node
    pub fn from_entity(entity: EntityId) -> Self {
        Self(entity)
    }
    /// Delete this node and its children
    pub fn despawn(self) {
        for c in entity::get_component(self.0, children()).unwrap_or_default() {
            Self::from_entity(c).despawn();
        }
        entity::despawn(self.0);
    }
}

/// Play mode for animation nodes
pub enum PlayMode {
    /// Play the animation
    Play {
        /// Global time for the start of the animation
        start_time: Duration,
    },
    /// Freeze the animation at time
    FreezeAtTime {
        /// Time in the animation we want to freeze at
        time: f32,
    },
    /// Freeze the animation at time = percentage * duration
    FreezeAtPercentage {
        /// Percentage into the animation that we want to freeze at
        percentage: f32,
    },
}
impl PlayMode {
    /// Play an animation
    pub fn play() -> Self {
        Self::Play {
            start_time: epoch_time(),
        }
    }
}

/// Play clip from url animation node.
/// This is an animation node which can be plugged into an animation player or other animation nodes.
///
/// This is just a reference to an entity which lives in the ecs. You need to call `despawn` to
/// remove it.
#[derive(Debug, Clone, Copy)]
pub struct PlayClipFromUrlNodeRef(pub AnimationNodeRef);
impl PlayClipFromUrlNodeRef {
    /// Create a new node.
    pub fn new(url: impl Into<String>) -> Self {
        let node = Entity::new()
            .with(play_clip_from_url(), url.into())
            .with(name(), "Play clip from URL".to_string())
            .with(looping(), true)
            .with(start_time(), epoch_time())
            .spawn();
        Self(AnimationNodeRef(node))
    }
    /// Use an existing node
    pub fn from_entity(entity: EntityId) -> Self {
        Self(AnimationNodeRef::from_entity(entity))
    }
    /// Set if the animation should loop or not
    pub fn looping(&self, value: bool) {
        entity::add_component(self.0 .0, looping(), value);
    }
    /// Set how the animation is played
    pub fn set_play_mode(&self, mode: PlayMode) {
        match mode {
            PlayMode::Play { start_time: time } => {
                entity::add_component(self.0 .0, start_time(), time);
                entity::remove_component(self.0 .0, freeze_at_time());
                entity::remove_component(self.0 .0, freeze_at_percentage());
            }
            PlayMode::FreezeAtTime { time } => {
                entity::add_component(self.0 .0, freeze_at_time(), time);
                entity::remove_component(self.0 .0, start_time());
                entity::remove_component(self.0 .0, freeze_at_percentage());
            }
            PlayMode::FreezeAtPercentage { percentage } => {
                entity::add_component(self.0 .0, freeze_at_percentage(), percentage);
                entity::remove_component(self.0 .0, start_time());
                entity::remove_component(self.0 .0, freeze_at_time());
            }
        }
    }
    /// Start playing the animation from the beginning
    pub fn restart(&self) {
        self.set_play_mode(PlayMode::Play {
            start_time: epoch_time(),
        });
    }
    /// Set up retargeting
    pub fn set_retargeting(&self, retargeting: AnimationRetargeting) {
        match retargeting {
            AnimationRetargeting::None => {
                entity::remove_component(self.0 .0, retarget_model_from_url());
            }
            AnimationRetargeting::Skeleton { model_url } => {
                entity::remove_component(self.0 .0, retarget_animation_scaled());
                entity::add_component(self.0 .0, retarget_model_from_url(), model_url);
            }
            AnimationRetargeting::AnimationScaled {
                normalize_hip,
                model_url,
            } => {
                entity::add_component(self.0 .0, retarget_animation_scaled(), normalize_hip);
                entity::add_component(self.0 .0, retarget_model_from_url(), model_url);
            }
        }
    }
    /// If true, the base pose from the model of the animation clip will be applied to the animation
    ///
    /// Some animations will only work if the base pose of the character is the same as
    /// the animations base pose, so we apply the pose from the animations model to make sure they
    /// correspond
    ///
    /// This exists mostly because some FBX animations have pre-rotations, and to apply them to
    /// character models which don't have the same pre-rotations we need to make sure they're up to sync
    ///
    /// I.e. this is mostly relevant for retargeting
    pub fn apply_base_pose(&self, value: bool) {
        if value {
            entity::add_component(self.0 .0, apply_base_pose(), ());
        } else {
            entity::remove_component(self.0 .0, apply_base_pose());
        }
    }
    /// Returns None if the duration hasn't been loaded yet
    pub fn peek_clip_duration(&self) -> Option<f32> {
        entity::get_component(self.0 .0, clip_duration())
    }
    /// Returns the duration of this clip. This is async because it needs to wait for the clip to load before the duration can be returned.
    pub async fn clip_duration(&self) -> f32 {
        entity::wait_for_component(self.0 .0, clip_duration())
            .await
            .unwrap_or_default()
    }
    /// Returns None if the clip hasn't been loaded yet
    pub fn peek_bind_ids(&self) -> Option<Vec<String>> {
        entity::get_component(self.0 .0, bind_ids())
    }
    /// Returns the bind ids of this clip. This is async because it needs to wait for the clip to load before the bind ids can be returned.
    pub async fn bind_ids(&self) -> Vec<String> {
        entity::wait_for_component(self.0 .0, bind_ids())
            .await
            .unwrap_or_default()
    }
    /// Wait until the clip has been loaded
    pub async fn wait_for_load(&self) {
        self.clip_duration().await;
    }
}
impl AsRef<AnimationNodeRef> for PlayClipFromUrlNodeRef {
    fn as_ref(&self) -> &AnimationNodeRef {
        &self.0
    }
}

/// Blend animation node.
/// This is an animation node which can be plugged into an animation player or other animation nodes.
///
/// This is just a reference to an entity which lives in the ecs. You need to call `despawn` to
/// remove it.
#[derive(Debug, Clone, Copy)]
pub struct BlendNodeRef(pub AnimationNodeRef);
impl BlendNodeRef {
    /// Create a new blend animation node.
    ///
    /// If the weight is 0, only the left animation will play.
    /// If the weight is 1, only the right animation will play.
    /// Values in between blend between the two animations.
    pub fn new(
        left: impl AsRef<AnimationNodeRef>,
        right: impl AsRef<AnimationNodeRef>,
        weight: f32,
    ) -> Self {
        let left: &AnimationNodeRef = left.as_ref();
        let right: &AnimationNodeRef = right.as_ref();
        let node = Entity::new()
            .with(blend(), weight)
            .with(name(), "Blend".to_string())
            .with(children(), vec![left.0, right.0])
            .spawn();
        entity::add_component(left.0, parent(), node);
        entity::add_component(right.0, parent(), node);
        Self(AnimationNodeRef(node))
    }
    /// Use an existing node
    pub fn from_entity(entity: EntityId) -> Self {
        Self(AnimationNodeRef::from_entity(entity))
    }
    /// Set the weight of this blend node.
    ///
    /// If the weight is 0, only the left animation will play.
    /// If the weight is 1, only the right animation will play.
    /// Values in between blend between the two animations.
    pub fn set_weight(&self, weight: f32) {
        entity::set_component(self.0 .0, blend(), weight);
    }
    /// Sets the mask of this blend node.
    ///
    /// For example `blend_node.set_mask(vec![("LeftLeg".to_string(), 1.)])` means
    /// that the LeftLeg is always controlled by the right animation.
    pub fn set_mask(&self, weights: Vec<(BindId, f32)>) {
        let (bind_ids, weights): (Vec<_>, Vec<_>) = weights
            .into_iter()
            .map(|(a, b)| (a.as_str().to_string(), b))
            .unzip();
        entity::add_component(self.0 .0, mask_bind_ids(), bind_ids);
        entity::add_component(self.0 .0, mask_weights(), weights);
    }
    /// Sets a mask value to all bones of a humanoids lower body
    pub fn set_mask_humanoid_lower_body(&self, weight: f32) {
        self.set_mask(
            BindId::HUMANOID_LOWER_BODY
                .iter()
                .map(|x| ((*x).clone(), weight))
                .collect(),
        );
    }
    /// Sets a mask value to all bones of a humanoids upper body
    pub fn set_mask_humanoid_upper_body(&self, weight: f32) {
        self.set_mask(
            BindId::HUMANOID_UPPER_BODY
                .iter()
                .map(|x| ((*x).clone(), weight))
                .collect(),
        );
    }
}
impl AsRef<AnimationNodeRef> for BlendNodeRef {
    fn as_ref(&self) -> &AnimationNodeRef {
        &self.0
    }
}

/// Animation retargeting configuration.
#[derive(Debug, Clone)]
pub enum AnimationRetargeting {
    /// Bone Translation comes from the animation data, unchanged.
    None,
    /// Bone Translation comes from the Target Skeleton's bind pose.
    Skeleton {
        /// Model url which the animation will be retargeted to.
        model_url: String,
    },
    /// Bone translation comes from the animation data, but is scaled by the Skeleton's proportions.
    /// This is the ratio between the bone length of the Target Skeleton (the skeleton the animation
    /// is being played on), and the Source Skeleton (the skeleton the animation was authored for).
    AnimationScaled {
        /// Rotates the Hips bone based on the difference between the rotation the animation models root and the retarget animations root
        normalize_hip: bool,
        /// Model url which the animation will be retargeted to.
        model_url: String,
    },
}
impl Default for AnimationRetargeting {
    fn default() -> Self {
        Self::None
    }
}

/// Get the bone entity from the bind_id; for example "LeftFoot"
pub fn get_bone_by_bind_id(entity: EntityId, bind_id: &BindId) -> Option<EntityId> {
    if let Some(bid) = entity::get_component(entity, self::bind_id()) {
        if bid == bind_id.as_str() {
            return Some(entity);
        }
    }
    if let Some(childs) = entity::get_component(entity, children()) {
        for c in childs {
            if let Some(bid) = get_bone_by_bind_id(c, bind_id) {
                return Some(bid);
            }
        }
    }
    None
}

/// Valid bind ids
#[derive(Debug, Clone)]
pub enum BindId {
    // Lower body for convenience
    /// Hips
    Hips,
    /// LeftFoot
    LeftFoot,
    /// LeftLeg
    LeftLeg,
    /// LeftToeBase
    LeftToeBase,
    /// LeftUpLeg
    LeftUpLeg,
    /// RightFoot
    RightFoot,
    /// RightLeg
    RightLeg,
    /// RightToeBase
    RightToeBase,
    /// RightUpLeg
    RightUpLeg,
    // Upper
    /// Head
    Head,
    /// LeftArm
    LeftArm,
    /// LeftForeArm
    LeftForeArm,
    /// LeftHand
    LeftHand,
    /// LeftHandIndex1
    LeftHandIndex1,
    /// LeftHandIndex2
    LeftHandIndex2,
    /// LeftHandIndex3
    LeftHandIndex3,
    /// LeftHandMiddle1
    LeftHandMiddle1,
    /// LeftHandMiddle2
    LeftHandMiddle2,
    /// LeftHandMiddle3
    LeftHandMiddle3,
    /// LeftHandPinky1
    LeftHandPinky1,
    /// LeftHandPinky2
    LeftHandPinky2,
    /// LeftHandPinky3
    LeftHandPinky3,
    /// LeftHandRing1
    LeftHandRing1,
    /// LeftHandRing2
    LeftHandRing2,
    /// LeftHandRing3
    LeftHandRing3,
    /// LeftHandThumb1
    LeftHandThumb1,
    /// LeftHandThumb2
    LeftHandThumb2,
    /// LeftHandThumb3
    LeftHandThumb3,
    /// LeftShoulder
    LeftShoulder,
    /// Neck
    Neck,
    /// RightArm
    RightArm,
    /// RightForeArm
    RightForeArm,
    /// RightHand
    RightHand,
    /// RightHandIndex1
    RightHandIndex1,
    /// RightHandIndex2
    RightHandIndex2,
    /// RightHandIndex3
    RightHandIndex3,
    /// RightHandMiddle1
    RightHandMiddle1,
    /// RightHandMiddle2
    RightHandMiddle2,
    /// RightHandMiddle3
    RightHandMiddle3,
    /// RightHandPinky1
    RightHandPinky1,
    /// RightHandPinky2
    RightHandPinky2,
    /// RightHandPinky3
    RightHandPinky3,
    /// RightHandRing1
    RightHandRing1,
    /// RightHandRing2
    RightHandRing2,
    /// RightHandRing3
    RightHandRing3,
    /// RightHandThumb1
    RightHandThumb1,
    /// RightHandThumb2
    RightHandThumb2,
    /// RightHandThumb3
    RightHandThumb3,
    /// RightShoulder
    RightShoulder,
    /// Spine
    Spine,
    /// Spine1
    Spine1,
    /// Spine2
    Spine2,

    /// Custom bind id
    Custom(String),
}
impl BindId {
    /// Get a string representation of this bind id
    pub fn as_str(&self) -> &str {
        match self {
            BindId::Hips => "Hips",
            BindId::LeftFoot => "LeftFoot",
            BindId::LeftLeg => "LeftLeg",
            BindId::LeftToeBase => "LeftToeBase",
            BindId::LeftUpLeg => "LeftUpLeg",
            BindId::RightFoot => "RightFoot",
            BindId::RightLeg => "RightLeg",
            BindId::RightToeBase => "RightToeBase",
            BindId::RightUpLeg => "RightUpLeg",
            BindId::Head => "Head",
            BindId::LeftArm => "LeftArm",
            BindId::LeftForeArm => "LeftForeArm",
            BindId::LeftHand => "LeftHand",
            BindId::LeftHandIndex1 => "LeftHandIndex1",
            BindId::LeftHandIndex2 => "LeftHandIndex2",
            BindId::LeftHandIndex3 => "LeftHandIndex3",
            BindId::LeftHandMiddle1 => "LeftHandMiddle1",
            BindId::LeftHandMiddle2 => "LeftHandMiddle2",
            BindId::LeftHandMiddle3 => "LeftHandMiddle3",
            BindId::LeftHandPinky1 => "LeftHandPinky1",
            BindId::LeftHandPinky2 => "LeftHandPinky2",
            BindId::LeftHandPinky3 => "LeftHandPinky3",
            BindId::LeftHandRing1 => "LeftHandRing1",
            BindId::LeftHandRing2 => "LeftHandRing2",
            BindId::LeftHandRing3 => "LeftHandRing3",
            BindId::LeftHandThumb1 => "LeftHandThumb1",
            BindId::LeftHandThumb2 => "LeftHandThumb2",
            BindId::LeftHandThumb3 => "LeftHandThumb3",
            BindId::LeftShoulder => "LeftShoulder",
            BindId::Neck => "Neck",
            BindId::RightArm => "RightArm",
            BindId::RightForeArm => "RightForeArm",
            BindId::RightHand => "RightHand",
            BindId::RightHandIndex1 => "RightHandIndex1",
            BindId::RightHandIndex2 => "RightHandIndex2",
            BindId::RightHandIndex3 => "RightHandIndex3",
            BindId::RightHandMiddle1 => "RightHandMiddle1",
            BindId::RightHandMiddle2 => "RightHandMiddle2",
            BindId::RightHandMiddle3 => "RightHandMiddle3",
            BindId::RightHandPinky1 => "RightHandPinky1",
            BindId::RightHandPinky2 => "RightHandPinky2",
            BindId::RightHandPinky3 => "RightHandPinky3",
            BindId::RightHandRing1 => "RightHandRing1",
            BindId::RightHandRing2 => "RightHandRing2",
            BindId::RightHandRing3 => "RightHandRing3",
            BindId::RightHandThumb1 => "RightHandThumb1",
            BindId::RightHandThumb2 => "RightHandThumb2",
            BindId::RightHandThumb3 => "RightHandThumb3",
            BindId::RightShoulder => "RightShoulder",
            BindId::Spine => "Spine",
            BindId::Spine1 => "Spine1",
            BindId::Spine2 => "Spine2",
            BindId::Custom(string) => string,
        }
    }
    /// Bind-ids for a humanoid's lower body
    pub const HUMANOID_LOWER_BODY: [BindId; 9] = [
        BindId::Hips,
        BindId::LeftFoot,
        BindId::LeftLeg,
        BindId::LeftToeBase,
        BindId::LeftUpLeg,
        BindId::RightFoot,
        BindId::RightLeg,
        BindId::RightToeBase,
        BindId::RightUpLeg,
    ];

    /// Bind-ids for a humanoid's upper body
    pub const HUMANOID_UPPER_BODY: [BindId; 43] = [
        BindId::Head,
        BindId::LeftArm,
        BindId::LeftForeArm,
        BindId::LeftHand,
        BindId::LeftHandIndex1,
        BindId::LeftHandIndex2,
        BindId::LeftHandIndex3,
        BindId::LeftHandMiddle1,
        BindId::LeftHandMiddle2,
        BindId::LeftHandMiddle3,
        BindId::LeftHandPinky1,
        BindId::LeftHandPinky2,
        BindId::LeftHandPinky3,
        BindId::LeftHandRing1,
        BindId::LeftHandRing2,
        BindId::LeftHandRing3,
        BindId::LeftHandThumb1,
        BindId::LeftHandThumb2,
        BindId::LeftHandThumb3,
        BindId::LeftShoulder,
        BindId::Neck,
        BindId::RightArm,
        BindId::RightForeArm,
        BindId::RightHand,
        BindId::RightHandIndex1,
        BindId::RightHandIndex2,
        BindId::RightHandIndex3,
        BindId::RightHandMiddle1,
        BindId::RightHandMiddle2,
        BindId::RightHandMiddle3,
        BindId::RightHandPinky1,
        BindId::RightHandPinky2,
        BindId::RightHandPinky3,
        BindId::RightHandRing1,
        BindId::RightHandRing2,
        BindId::RightHandRing3,
        BindId::RightHandThumb1,
        BindId::RightHandThumb2,
        BindId::RightHandThumb3,
        BindId::RightShoulder,
        BindId::Spine,
        BindId::Spine1,
        BindId::Spine2,
    ];
}
