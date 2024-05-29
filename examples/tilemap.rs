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
        app.renderer.add_tilemap_renderer();

        Self
    }

    fn update(&mut self, app: &mut App, event: &StateEvent)
    {
        
    }
}