use app::*;

fn main()
{
    baguette::new()
        .add_loop::<AppWithTileMap>()
        .run()
}

struct AppWithTileMap;

impl State for AppWithTileMap
{
    fn new(app: &mut App) -> Self where Self: Sized
    {
        app.renderer.add_tilemap
        (
            TilemapBuilder::default()
                .add_texture("assets/green dude.png", 1, 1)
        );

        Self
    }

    fn update(&mut self, _: &mut App, _: &StateEvent)
    {
        
    }
}