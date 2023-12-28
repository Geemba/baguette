use app::*;

fn main()
{
    baguette_core::new()
        .add_loop::<TestA>()
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
                    path: r"D:\Rust\baguette\assets\melastrana green sheet.png",
                    filtermode: FilterMode::Nearest,
                    instances: vec!
                    [
                        (
                            Default::default(),
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
    }
}