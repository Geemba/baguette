<p align="center">
  <img src="assets/baguette_logo_and_text.png"/>

<h2 align="center">🥖🥖A rust baked game engine 🥖🥖</h2>
  
<h4 align="center">note: this crate is being used for a game i'm working on, i will keep updating this engine as more content will eventually be required </h4>



## Roadmap

- [x] Input
    - [x] Keyboard
    - [x] Mouse
    - [ ] Controller
          
- [ ] Audio player
    - [x] basic playback
          
- [ ] Rendering
    - [x] 3d Camera
       - [x] Translation
       - [x] Orientation with Quaternions
       - [x] Zooming
       - [x] Perspective / Orthographic
       - [ ] .

    - [x] Sprite Rendering
       - [x] Gpu instancing
       - [x] SpriteSheet Animations
    - [x] Fullscreen
    - [x] Screen Resizing
    - [x] Window Icon
        
- [x] Egui integration

- [ ] Coroutines
- [ ] Tweening

# example:

a small scene with a camera and an animated sprite from a spritesheet

```bash
use app::*; //prelude

fn main()
{
    baguette_core::new()
        // add a non exiting loop to run
        .add_loop::<Test>()
        .run()
}

struct TestA
{
    time: u8,
    cam: &'static mut Camera,
    sprite: Sprite,
}

impl State for TestA
{
    fn new(app: &mut Application) -> Self where Self: Sized
    {
        Self
        {
            time: 0,
            cam: Camera::main_mut(),
            sprite: app.renderer.load_sprite
            (
                SpriteLoader::SpriteSheet
                {
                    path: r"..\melastrana green sheet.png",
                    filtermode: FilterMode::Nearest,
                    instances: vec!
                    [
                        (
                            Transform { translation: math::Vec3::X * -1., ..Default::default() },
                            SheetTiles::RangeIn(19..=21)
                        )
                    ],
                    pxunit: 100.,
                    layout: SpriteLayout { rows: 6, columns: 5 }
                }
            )
        }
    }

    fn update(&mut self, _: &StateEvent)
    {
        self.move_cam();

        match self.time > 6
        {
            true =>
            {
                for instance in self.sprite.iter_instances_mut()
                {
                    instance.section.next_or_first();
                }

                self.time = 0
            }
            false => self.time += 1
        }
    }
}

impl TestA
{
    fn move_cam(&mut self)
    {
        if input::get_key_holding(input::KeyCode::KeyW)
        {
            self.cam.set_position(self.cam.position() + (math::Vec3::Z * -0.1))
        }
        if input::get_key_holding(input::KeyCode::KeyS)
        {
            self.cam.set_position(self.cam.position() + (math::Vec3::Z * 0.1))
        }
        if input::get_key_holding(input::KeyCode::KeyA)
        {
            self.cam.set_position(self.cam.position() + (math::Vec3::X * -0.1))
        }
        if input::get_key_holding(input::KeyCode::KeyD)
        {
            self.cam.set_position(self.cam.position() + (math::Vec3::X * 0.1))
        }

        if input::get_key_holding(input::KeyCode::ArrowUp)
        {
            self.cam.rotate(math::math::Quat::from_axis_angle(math::Vec3::X, 1f32.to_radians()))
        }
        if input::get_key_holding(input::KeyCode::ArrowDown)
        {
            self.cam.rotate(math::math::Quat::from_axis_angle(math::Vec3::X, -1f32.to_radians()))
        }
        if input::get_key_holding(input::KeyCode::ArrowLeft)
        {
            self.cam.rotate(math::math::Quat::from_axis_angle(math::Vec3::Y, 1f32.to_radians()))
        }
        if input::get_key_holding(input::KeyCode::ArrowRight)
        {
            self.cam.rotate(math::math::Quat::from_axis_angle(math::Vec3::Y, -1f32.to_radians()))
        }
    }
}
```

## If you are looking for rust game engines..
🚨 NB: if you are looking for mature and stable game engines rust is not the correct place,
all of them are still in their infancy although they are promising engines worth trying. 🚨

- [Bevy](https://github.com/bevyengine/bevy) - The bevy engine
- [Macroquad](https://github.com/not-fl3/macroquad) - is a simple and easy to use game library for Rust programming language, heavily inspired by raylib.

## License

MIT, Apache 2.0

---

> GitHub [Geemba](https://github.com/Geemba)

