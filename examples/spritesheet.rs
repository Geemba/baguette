use app::*;

fn main()
{
    baguette::new()
        .add_state::<TestA>()
        .run()
}

struct TestA
{
    time: u8,
    sprite: SpriteSheet,
}

impl AppState for TestA
{
    fn new(app: &mut App) -> Self
    {
        Self
        {
            time: 0,
            sprite: SpriteSheet::new
            (
                &mut app.renderer,
                SpriteSheetBuilder::new
                (
                    "assets/green dude sheet.png",
                    6, 5
                )
                .set_layer::<0>
                (
                    [(Default::default(), SheetSlices::Range(19..22))]
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
                for mut section in self.sprite.iter_layer_mut(0)
                {
                    section.next_or_first();
                }

                self.time = 0
            }
            false => self.time += 1
        }
    }
}