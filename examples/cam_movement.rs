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
    sprite: SpriteSheet,
}

impl State for TestA
{
    fn new(app: &mut App) -> Self where Self: Sized
    {
        Self
        {
            timer: 0,
            cam: Camera::get(&mut app.renderer),
            sprite: SpriteSheet::new
            (
                &mut app.renderer,
                SpriteSheetLoader::new_pixelated(r"assets\green dude sheet.png", [], 6, 5)
            )
        }
    }

    fn update(&mut self, app: &mut App, _: &StateEvent)
    {
        self.move_cam(&app.input);

        match self.timer == 0
        {
            true =>
            {
                for (.., section) in self.sprite.iter_mut()
                {
                    section.next_or_first();
                }

                self.timer = 8
            }
            false => self.timer -= 1
        }
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