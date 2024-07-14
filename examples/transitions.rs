use app::*;

/// this example shows transition capabilities between states by pressing the return key (enter)
fn main()
{
    baguette::new()
        .add_state::<State1>()
        .add_state::<State2>()
        .run()
}

struct State1
{
    time: u8,
    sprite: SpriteSheet
}

impl AppState for State1
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

    fn transitions(&self, app: &App) -> Option<StateId>
    {
        if app.get_key_down(input::KeyCode::Enter)
        {
            return Some(State2::id())
        }

        None
    }
}

struct State2; impl AppState for State2
{
    fn new(app: &mut App) -> Self
    {
        Self
    }

    fn update(&mut self, _: &mut App, _: &StateEvent)
    {
        
    }

    fn transitions(&self, app: &App) -> Option<StateId>
    {
        if app.get_key_down(input::KeyCode::Enter)
        {
            return Some(State1::id())
        }

        None
    }
}