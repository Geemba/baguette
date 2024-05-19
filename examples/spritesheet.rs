use app::{spritesheet::SheetSlices, *};

fn main()
{
    baguette::new()
        .add_loop::<TestA>()
        .run()
}

struct TestA
{
    time: u8,
    sprite: SpriteSheet,
}

impl State for TestA
{
    fn new(app: &mut App) -> Self
    {
        Self
        {
            time: 0,
            sprite: SpriteSheet::new
            (
                &mut app.renderer,
                SpriteSheetLoader::new_pixelated
                (
                    "assets/green dude sheet.png",
                    [(Default::default(), SheetSlices::Range(19..22))],
                    6, 5
                )
            ),
        }
    }

    fn update(&mut self, _: &mut App, _: &StateEvent)
    {
        match self.time > 8
        {
            true =>
            {
                for (.., section) in self.sprite.iter_mut()
                {
                    section.next_or_first();
                }

                self.time = 0
            }
            false => self.time += 1
        }
    }
}