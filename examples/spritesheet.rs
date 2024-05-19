use app::*;

fn main()
{
    baguette::new()
        .add_loop::<TestA>()
        .run()
}

struct TestA
{
    time: u8,
    sprite: Sprite,
}

impl State for TestA
{
    fn new(app: &mut App) -> Self where Self: Sized
    {
        Self
        {
            time: 0,
            sprite: app.renderer.load_sprite
            (
                SpriteLoader::new(r"assets\green dude sheet.png",)
            )
        }
    }

    fn update(&mut self, _app: &mut App, _: &StateEvent)
    {
        todo!()
        //match self.time > 8
        //{
        //    true =>
        //    {
        //        for instance in self.sprite.iter_mut()
        //        {
        //            instance.section.next_or_first();
        //        }

        //        self.time = 0
        //    }
        //    false => self.time += 1
        //}
    }
}