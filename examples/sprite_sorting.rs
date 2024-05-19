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
                SpriteLoader::new_pixelated(r"assets\baguette.png")
                    //.pivot(pivot)
            ),
            sprite2: Sprite::new
            (
                &mut app.renderer,
                SpriteLoader::new_pixelated(r"assets\green dude.png")
            ),
            go_up: true,
        }
    }

    fn update(&mut self, _: &mut App, _: &StateEvent)
    {
        for sprite in self.sprite2.iter_mut()
        {
            let height = sprite.translation.y;

            if height > 0.2
            {
                self.go_up = false
            }
            else if height < -0.2
            {
                self.go_up = true
            }
            
            sprite.translation.y += match self.go_up
            {
                true => 0.007,
                false => -0.007,
            }

        }
    }
}