# Common pitfalls

Be aware that a lot of problems are caused by mismatching versions of Ambient. To check your version, run `ambient --version` and make sure it matches the version in your `Cargo.toml` file.

## The examples don't work

This is most often because of mismatching the ambient version with the
examples version. See [running examples](../user/running_examples.md).

## My clientside WASM module crashes when accessing a component from the server and unwrapping it

Your clientside WASM can run before the server has finished running its WASM, so the component you're trying to access may not have been created yet.

To fix this, consider using `entity::wait_for_component`, which is an async helper that will stall execution until the component is available.

## My object with a random color is black sometimes

The `color` component is a `Vec4`. Using `rand::random` to populate it will
result in the `w`/alpha channel also being between 0 and 1, which means your
object may be black and/or disappear if the alpha is below the default alpha
cut-off.

To fix this, use a random `Vec3` for your color and then extend it to a `Vec4`:

```rust
let color = rand::random::<Vec3>().extend(1.0);
```

## My character controller is unaffected by gravity

PhysX, which we use for physics, does not apply gravity to character controllers.

You can account for this by applying gravity to the character controller yourself;
an example of this can be found in [the `character_movement` standard package](https://github.com/AmbientRun/Ambient/blob/main/guest/rust/packages/std/character_movement/src/server.rs)
which maintains a `vertical_velocity` component and uses it to simulate gravity.

## My camera's view matrix is all NaNs

This can happen when the transformation used to position the camera in the world is invalid.

There are several potential causes, including:

- The camera is positioned at the origin, and is looking at the origin.
- The camera's `lookat_up` vector is parallel to the `lookat_target` vector. This can happen by default if your `lookat_target` is above or below the camera as `lookat_up` defaults to +Z.
- There is a division by zero somewhere in the camera's transformation. This could happen if your gameplay code for controlling the camera does not account for the possibility of a zero denominator (i.e. no time passing, or no distance travelled).

## Fails to start on Linux (Error in `Surface::configure: parent device is lost`)

If you're running Wayland, you may have to start Ambient with `WAYLAND_DISPLAY=wayland-1 ambient run`.
See [this issue](https://github.com/gfx-rs/wgpu/issues/2519) for details.

## Runtime error: import `...` has the wrong type

This can occur when you have `.wasm` files in your `build` folder that are using an old version of the Ambient API.
Delete the `build` folder and try again - this should force them to be regenerated.

## Failed to download file / error trying to connect: tcp connect error: _etc_ (OS error 10060)

This can happen if your anti-virus or firewall is blocking the connection to the Ambient runtime.
Try deactivating it, then run the Ambient package again with 'ambient run'.

If this fixes it, you'll need to add an exception to your anti-virus/firewall to allow Ambient to connect.
We do not recommend leaving your anti-virus/firewall disabled.

## `<ciso646>` not found

The compilation of `physx-sys` and other C++ libraries may fail due to a missing `ciso646` header.
This header was removed as part of C++20, and distributions no longer ship it by default.

This can be fixed on Debian-based distributions (i.e. Ubuntu 22.04, Pop!\_OS 22.04, etc) by running

```sh
sudo apt install libstdc++-12-dev
```

to install a version of the GNU C++ standard library that includes the header.
