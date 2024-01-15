use app::*;

/// a very simple ui test that shows how to display variables on a window,
/// read the egui docs to better understand how to use it
fn main()
{
    baguette::new()
        .add_loop::<TestA>()
        .run()
}

struct TestA
{
    cam: &'static mut Camera,
    sprite: Sprite,
    open_window: bool
}

impl State for TestA
{
    fn new(app: &mut App) -> Self where Self: Sized
    {
        Self
        {
            cam: Camera::main_mut(),
            sprite: app.renderer.load_sprite
            (
                SpriteLoader::Sprite
                {
                    path: r"assets\green dude.png",
                    filtermode: FilterMode::Nearest,
                    instances: vec![Transform::default()],
                    pxunit: 100.,
                }
            ),
            open_window: true,
        }
    }

    fn update(&mut self, app: &mut App<'_>, _: &StateEvent)
    {
        self.move_cam(&app.input);

        app.close();

        egui::Window::new("window example")
            .movable(true)
            .show(app.ui().context(), |ui|
            {
                let button = ui.button("close app");
                
                if button.clicked()
                {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }

                ui.label
                (
                    egui::RichText::new(self.cam.position().to_string()).size(30.)
                );

                ui.label
                (
                    egui::RichText::new("clicked:".to_owned() + &button.clicked().to_string()).size(30.)
                );

                ui.label
                (
                    egui::RichText::new("is egui responding :".to_owned() + &ui.ctx().wants_pointer_input().to_string()).size(30.)
                );
                
            });
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