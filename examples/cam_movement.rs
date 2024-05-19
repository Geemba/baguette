use app::*;

fn main()
{
    baguette::new()
        .add_loop::<TestA>()
        .run()
}

struct TestA
{
    timer: u8,
    cam: Camera,
    sprite: Sprite,
}

impl State for TestA
{
    fn new(app: &mut App) -> Self where Self: Sized
    {
        todo!()
        //Self
        //{
        //    timer: 0,
        //    cam: Camera::get(&mut app.renderer),
        //    sprite: app.renderer.load_sprite
        //    (
        //        SpriteLoader::SpriteSheet
        //        {
        //            path: r"assets\green dude sheet.png",
        //            filtermode: FilterMode::Nearest,
        //            instances: vec!
        //            [
        //                (
        //                    Transform
        //                    {
        //                        translation: (0.,0.,1.).into(),
        //                        ..Default::default()
        //                    },
        //                    SheetTiles::RangeIn(19..=21)
        //                )
        //            ],
        //            pxunit: 100.,
        //            layout: SpriteLayout { rows: 6, columns: 5 }
        //        }
        //    )
        //}
    }

    fn update(&mut self, app: &mut App, _: &StateEvent)
    {
        self.move_cam(&app.input);
        todo!()
        //match self.timer == 0
        //{
        //    true =>
        //    {
        //        for instance in self.sprite.iter_mut()
        //        {
        //            instance.section.next_or_first();
        //        }

        //        self.timer = 8
        //    }
        //    false => self.timer -= 1
        //}
    }
}

impl TestA
{
    fn move_cam(&mut self, input: &input::Input)
    {
        if input.get_key_holding(input::KeyCode::KeyW)
        {
            self.cam.set_position(self.cam.position() + (math::Vec3::Z * -0.1))
        }
        if input.get_key_holding(input::KeyCode::KeyS)
        {
            self.cam.set_position(self.cam.position() + (math::Vec3::Z * 0.1))
        }
        if input.get_key_holding(input::KeyCode::KeyA)
        {
            self.cam.set_position(self.cam.position() + (math::Vec3::X * -0.1))
        }
        if input.get_key_holding(input::KeyCode::KeyD)
        {
            self.cam.set_position(self.cam.position() + (math::Vec3::X * 0.1))
        }
    }
}