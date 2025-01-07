use std::time::Duration;

use crate::{
    global::{
        IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle, ProceduralMeshHandle,
        ProceduralSamplerHandle, ProceduralTextureHandle, Quat, UVec2, UVec3, UVec4, Vec2, Vec3,
        Vec4,
    },
    internal::wit,
};
use ambient_shared_types::procedural_storage_handle_definitions;
use paste::paste;

/// Converts from a Rust representation to a wit-bindgen representation.
pub trait IntoBindgen {
    type Item;
    fn into_bindgen(self) -> Self::Item;
}

/// Converts from a wit-bindgen representation to a Rust representation.
pub trait FromBindgen {
    type Item;

    #[allow(clippy::wrong_self_convention)]
    fn from_bindgen(self) -> Self::Item;
}

impl IntoBindgen for Vec2 {
    type Item = wit::types::Vec2;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Vec2 {
            x: self.x,
            y: self.y,
        }
    }
}
impl FromBindgen for wit::types::Vec2 {
    type Item = Vec2;
    fn from_bindgen(self) -> Self::Item {
        Vec2::new(self.x, self.y)
    }
}

impl IntoBindgen for Vec3 {
    type Item = wit::types::Vec3;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}
impl FromBindgen for wit::types::Vec3 {
    type Item = Vec3;
    fn from_bindgen(self) -> Self::Item {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl IntoBindgen for Vec4 {
    type Item = wit::types::Vec4;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Vec4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}
impl FromBindgen for wit::types::Vec4 {
    type Item = Vec4;
    fn from_bindgen(self) -> Self::Item {
        Vec4::new(self.x, self.y, self.z, self.w)
    }
}

impl IntoBindgen for UVec2 {
    type Item = wit::types::Uvec2;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Uvec2 {
            x: self.x,
            y: self.y,
        }
    }
}
impl FromBindgen for wit::types::Uvec2 {
    type Item = UVec2;
    fn from_bindgen(self) -> Self::Item {
        UVec2::new(self.x, self.y)
    }
}

impl IntoBindgen for UVec3 {
    type Item = wit::types::Uvec3;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Uvec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}
impl FromBindgen for wit::types::Uvec3 {
    type Item = UVec3;
    fn from_bindgen(self) -> Self::Item {
        UVec3::new(self.x, self.y, self.z)
    }
}

impl IntoBindgen for UVec4 {
    type Item = wit::types::Uvec4;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Uvec4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}
impl FromBindgen for wit::types::Uvec4 {
    type Item = UVec4;
    fn from_bindgen(self) -> Self::Item {
        UVec4::new(self.x, self.y, self.z, self.w)
    }
}

impl IntoBindgen for IVec2 {
    type Item = wit::types::Ivec2;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Ivec2 {
            x: self.x,
            y: self.y,
        }
    }
}
impl FromBindgen for wit::types::Ivec2 {
    type Item = IVec2;
    fn from_bindgen(self) -> Self::Item {
        IVec2::new(self.x, self.y)
    }
}

impl IntoBindgen for IVec3 {
    type Item = wit::types::Ivec3;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Ivec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}
impl FromBindgen for wit::types::Ivec3 {
    type Item = IVec3;
    fn from_bindgen(self) -> Self::Item {
        IVec3::new(self.x, self.y, self.z)
    }
}

impl IntoBindgen for IVec4 {
    type Item = wit::types::Ivec4;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Ivec4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}
impl FromBindgen for wit::types::Ivec4 {
    type Item = IVec4;
    fn from_bindgen(self) -> Self::Item {
        IVec4::new(self.x, self.y, self.z, self.w)
    }
}

impl IntoBindgen for Quat {
    type Item = wit::types::Quat;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Quat {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}
impl FromBindgen for wit::types::Quat {
    type Item = Quat;
    fn from_bindgen(self) -> Self::Item {
        Quat::from_array([self.x, self.y, self.z, self.w])
    }
}

impl IntoBindgen for Mat4 {
    type Item = wit::types::Mat4;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Mat4 {
            x: self.x_axis.into_bindgen(),
            y: self.y_axis.into_bindgen(),
            z: self.z_axis.into_bindgen(),
            w: self.w_axis.into_bindgen(),
        }
    }
}
impl FromBindgen for wit::types::Mat4 {
    type Item = Mat4;
    fn from_bindgen(self) -> Self::Item {
        Mat4::from_cols(
            self.x.from_bindgen(),
            self.y.from_bindgen(),
            self.z.from_bindgen(),
            self.w.from_bindgen(),
        )
    }
}

impl IntoBindgen for Duration {
    type Item = wit::types::Duration;
    fn into_bindgen(self) -> Self::Item {
        wit::types::Duration {
            seconds: self.as_secs(),
            nanoseconds: self.subsec_nanos(),
        }
    }
}
impl FromBindgen for wit::types::Duration {
    type Item = Duration;
    fn from_bindgen(self) -> Self::Item {
        Duration::new(self.seconds, self.nanoseconds)
    }
}

macro_rules! bindgen_passthrough {
    ($type:ty) => {
        impl IntoBindgen for $type {
            type Item = Self;
            fn into_bindgen(self) -> Self::Item {
                self
            }
        }
        impl FromBindgen for $type {
            type Item = Self;
            fn from_bindgen(self) -> Self::Item {
                self
            }
        }
    };
}

bindgen_passthrough!(bool);
bindgen_passthrough!(f32);
bindgen_passthrough!(f64);
bindgen_passthrough!(String);
bindgen_passthrough!(u8);
bindgen_passthrough!(u16);
bindgen_passthrough!(u32);
bindgen_passthrough!(u64);
bindgen_passthrough!(i8);
bindgen_passthrough!(i16);
bindgen_passthrough!(i32);
bindgen_passthrough!(i64);

impl<T> IntoBindgen for Option<T>
where
    T: IntoBindgen,
{
    type Item = Option<T::Item>;
    fn into_bindgen(self) -> Self::Item {
        self.map(|i| i.into_bindgen())
    }
}
impl<T> FromBindgen for Option<T>
where
    T: FromBindgen,
{
    type Item = Option<T::Item>;
    fn from_bindgen(self) -> Self::Item {
        self.map(|i| i.from_bindgen())
    }
}

impl<T> IntoBindgen for Vec<T>
where
    T: IntoBindgen,
{
    type Item = Vec<T::Item>;
    fn into_bindgen(self) -> Self::Item {
        self.into_iter().map(|i| i.into_bindgen()).collect()
    }
}
impl<T> FromBindgen for Vec<T>
where
    T: FromBindgen,
{
    type Item = Vec<T::Item>;
    fn from_bindgen(self) -> Self::Item {
        self.into_iter().map(|i| i.from_bindgen()).collect()
    }
}

macro_rules! make_procedural_storage_handle_converters {
    ($($name:ident),*) => { paste!{$(
        impl FromBindgen for wit::[<client_ $name>]::Handle {
            type Item = [<Procedural $name:camel Handle>];

            fn from_bindgen(self) -> Self::Item {
                [<Procedural $name:camel Handle>](self.ulid)
            }
        }

        impl IntoBindgen for [<Procedural $name:camel Handle>] {
            type Item = wit::[<client_ $name>]::Handle;

            fn into_bindgen(self) -> Self::Item {
                Self::Item { ulid: self.0 }
            }
        }
    )*}};
}

procedural_storage_handle_definitions!(make_procedural_storage_handle_converters);
