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
                SpriteLoader::SpriteSheet
                {
                    path: r"assets\green dude sheet.png",
                    filtermode: FilterMode::Nearest,
                    instances: vec!
                    [
                        (
                            Transform
                            {
                                translation: (0.,0.,1.).into(),
                                ..Default::default()
                            },
                            SheetTiles::RangeIn(19..=21)
                        )
                    ],
                    pxunit: 100.,
                    layout: SpriteLayout { rows: 6, columns: 5 }
                }
            )
        }
    }

    fn update(&mut self, app: &mut App<'_>, _: &StateEvent)
    {
        match self.time > 8
        {
            true =>
            {
                for instance in self.sprite.iter_instances_mut()
                {
                    instance.section.next_or_first();
                }

                self.time = 0
            }
            false => self.time += 1
        }
    }
}