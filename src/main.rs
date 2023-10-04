use gameloop::*;

fn main()
{
    baguette_core::new()
    .set_theme(baguette_core::WindowTheme::Dark)
    .add_state::<TestA>(transitions!
    [
        |_s| input::get_key_down(input::KeyCode::Return) => TestB
    ])
    .add_state::<TestB>(transitions!
    (
        |_s| input::get_key_down(input::KeyCode::Return) => TestA
    ))
    .run()
}

struct TestB;

impl State for TestB
{
    fn new() -> Self where Self : Sized
    {
        Self
    }

    fn update(&mut self, _ : &StateEvent)
    {
        
    }
}

struct TestA
{
    cam : &'static mut rendering::Camera,
    audio_player : audio::AudioPlayer
}

impl State for TestA
{
    fn new() -> Self where Self : Sized 
    {
        TestA { cam : rendering::camera::main(), audio_player: audio::AudioPlayer::new(4) }
    }

    fn update(&mut self, event : &gameloop::StateEvent)
    {
        if let gameloop::StateEvent::Update = event
        {
            if input::get_key_holding(input::KeyCode::W)
            {
                self.cam.set_position(self.cam.position() + (math::Vec3::Z * -0.5))
            }
            if input::get_key_holding(input::KeyCode::S)
            {
                self.cam.set_position(self.cam.position() + (math::Vec3::Z * 0.5))
            }
            if input::get_key_down(input::KeyCode::A)
            {
                self.audio_player.play(r"C:\Users\danid\OneDrive\Desktop\MI_SFX 04.mp3");
            }
        }
    }
}