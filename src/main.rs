use app::*;

fn main()
{
    baguette_core::new()
    
        .add_state::<Empty>(transitions!
        [
            |_| input::get_key_down(input::KeyCode::Enter) => TestA
        ])
        .add_state::<TestA>(transitions!
        [
            |_| input::get_key_down(input::KeyCode::Enter) => Empty
        ])
        .run()
}

struct Empty;

impl State for Empty
{
    fn new(_ : &'static mut Application) -> Self where Self: Sized
    {
        Self
    }

    fn update(&mut self, _ : &StateEvent)
    {

    }
}

struct TestA
{
    time: u8,
    cam: &'static mut Camera,
    sprite: Sprite,
    sprite2: Sprite
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
                    path: r"D:\Fruit_Dungeon\baguette\assets\melastrana green sheet.png",
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
            ),
            sprite2: app.renderer.load_sprite
            (
                SpriteLoader::SpriteSheet
                {
                    path: r"D:\Fruit_Dungeon\baguette\assets\melastrana green sheet.png",
                    filtermode: FilterMode::Nearest,
                    instances: vec!
                    [
                        (
                            Transform { translation: math::Vec3::X * 1., ..Default::default() },
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

        if input::get_key_down(input::KeyCode::Backspace)
        {
            self.cam.set_projection_mode(ProjectionMode::Orthographic)
        }

        match self.time > 6
        {
            true =>
            {
                for instance in self.sprite.iter_instances_mut()
                {
                    instance.section.next_or_first();
                }
                for instance in self.sprite2.iter_instances_mut()
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