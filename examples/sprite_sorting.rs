use app::*;

fn main()
{
    baguette::new()
        .add_loop::<Application>()
        .run()
}

struct Application
{
    sprite: Sprite,
    sprite2: Sprite,
    go_up: bool
}

impl State for Application
{
    fn new(app: &mut App) -> Self where Self: Sized
    {
        Self
        {
            sprite: Sprite::new
            (
                &mut app.renderer,
                SpriteBuilder::new(r"assets\baguette.png")
                    .pivot((0., -0.1))
            ),
            sprite2: Sprite::new
            (
                &mut app.renderer,
                SpriteBuilder::new(r"assets\green dude.png")
                    .pivot((0., -0.1))
            ),
            go_up: true,
        }
    }

    fn update(&mut self, _: &mut App, _: &StateEvent)
    {
        for sprite in self.sprite2.iter_layer_mut(0)
        {
            let height = sprite.translation.y;

            if height > 0.1
            {
                self.go_up = false
            }
            else if height < -0.1
            {
                self.go_up = true
            }
            
            sprite.translation.y += match self.go_up
            {
                true => 0.001,
                false => -0.001,
            }

        }
    }
}