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
    fn new(app: &mut App) -> Self
    {
        app.renderer.add_tilemap
        (
            TilemapBuilder::default()
                .add_layer::<0>
                (
                    [
                        Tile{ pos: (0.0 ,0.1).into(), ..Default::default() },
                        Tile{ pos: (0.9 ,0.1).into(), ..Default::default() }
                    ]
                )
                .add_texture("assets/green dude.png", 1, 1)
                
        );

        Self
    }

    fn update(&mut self, _: &mut App, _: &StateEvent)
    {
        
    }
}