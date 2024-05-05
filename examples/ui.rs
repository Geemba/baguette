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
    cam: Camera,
    sprite: Sprite,
}

impl State for TestA
{
    fn new(app: &mut App) -> Self where Self: Sized
    {
        Self
        {
            cam: Camera::get(&mut app.renderer),
            sprite: app.renderer.load_sprite
            (
                SpriteLoader::SingleSprite
                {
                    path: r"assets\green dude.png",
                    filtermode: FilterMode::Nearest,
                    instances: vec![Transform::default()],
                    pxunit: 100.,
                }
            ),
        }
    }

    fn update(&mut self, app: &mut App, _: &StateEvent)
    {
        self.move_cam(&app.input);

        ui::Window::new("window example")
            .show(app.ui().context(), |ui|
            {
                ui.group
                (
                    |ui|
                    {
                        ui.label
                        (
                            ui::RichText::new
                            (
                                "Camera position: \n".to_owned() +
                                &format!
                                (
                                    "{:.1}, {:.1}, {:.1}",
                                    
                                    self.cam.position().x,
                                    self.cam.position().y,
                                    self.cam.position().z
                                )
                            )
                                .monospace()
                                .size(20.)
                        );

                        if ui.button(ui::RichText::new("reset").size(20.)).clicked()
                        {
                            self.cam.set_position(math::Vec3::Z * 2.)
                        }
                    }
                );

                let button = ui.button
                (
                    ui::RichText::new("Close App")
                        .size(30.)
                );

                if button.clicked()
                {
                    ui.ctx().send_viewport_cmd(ui::ViewportCommand::Close);
                }                              
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