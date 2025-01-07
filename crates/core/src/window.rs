use ambient_ecs::{components, Description, Name, Resource, World};
use ambient_native_std::math::interpolate;
use glam::{uvec2, vec2, UVec2, Vec2, Vec3, Vec3Swizzles};
use winit::window::{CursorGrabMode, CursorIcon, Window};

pub use ambient_ecs::generated::app::components::{
    cursor_position, window_logical_size, window_physical_size, window_scale_factor,
};

components!("app", {
    @[Resource, Name["Window Control"], Description["Allows controlling the window from afar."]]
    window_ctl: flume::Sender<WindowCtl>,
});

pub fn set_cursor(world: &World, cursor: CursorIcon) {
    world
        .resource(window_ctl())
        .send(WindowCtl::SetCursorIcon(cursor))
        .ok();
}

pub fn screen_to_clip_space(world: &World, screen_pos: Vec2) -> Vec2 {
    let screen_size = *world.resource(window_logical_size());
    interpolate(
        screen_pos,
        Vec2::ZERO,
        screen_size.as_vec2(),
        vec2(-1., 1.),
        vec2(1., -1.),
    )
}
pub fn clip_to_screen_space(world: &World, clip_pos: Vec3) -> Vec3 {
    let screen_size = *world.resource(window_logical_size());
    interpolate(
        clip_pos.xy(),
        vec2(-1., 1.),
        vec2(1., -1.),
        Vec2::ZERO,
        screen_size.as_vec2(),
    )
    .extend(clip_pos.z)
}
pub fn get_mouse_clip_space_position(world: &World) -> Vec2 {
    let mouse_position = *world.resource(cursor_position());
    screen_to_clip_space(world, mouse_position)
}

pub fn get_window_sizes(window: &Window) -> (UVec2, UVec2, f64) {
    let size = uvec2(window.inner_size().width, window.inner_size().height);
    let sf = window.scale_factor();
    (size, (size.as_dvec2() / sf).as_uvec2(), sf)
}

/// Allows controlling the window
#[derive(Debug, Clone)]
pub enum WindowCtl {
    GrabCursor(CursorGrabMode),
    SetCursorIcon(CursorIcon),
    ShowCursor(bool),
    SetTitle(String),
    SetFullscreen(bool),
    ExitProcess(ExitStatus),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ExitStatus(i32);

impl ExitStatus {
    pub const SUCCESS: Self = ExitStatus(0);
    pub const FAILURE: Self = ExitStatus(1);
}
