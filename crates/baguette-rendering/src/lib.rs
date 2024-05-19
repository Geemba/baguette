//! rendering module of baguette

pub use input::winit::window::Window;

pub(crate) use baguette_math::*;

/// image crate reexport
pub use image;

#[path ="rendering/2d/sprite.rs"]
pub mod sprite;
pub use sprite::Sprite;
pub use sprite::SpriteLayout;
pub use sprite::SpriteInstance;

#[path ="rendering/2d/spritesheet.rs"]
pub mod spritesheet;
pub use spritesheet::SpriteSheet;
pub use spritesheet::SpriteSheetLoader;

#[path ="rendering/renderer.rs"]
pub mod renderer;
pub use renderer::*;

#[path ="rendering/ui/ui.rs"]
pub mod ui;
//pub use ui::*;

#[path ="rendering/renderpasses.rs"]
mod renderpasses;
pub use renderpasses::*;

#[path = "rendering/2d/spritepass.rs"]
pub mod spritepass;
pub use spritepass::*;

#[path ="rendering/camera.rs"]
pub mod camera;
pub use camera::*;

#[path ="rendering/raw/transform.rs"]
pub mod transform;
#[path ="rendering/raw/vertex.rs"]
pub mod vertex;
#[path ="rendering/raw/texture.rs"]
pub mod texture;

#[path ="rendering/renderer/util.rs"]
pub mod util;

pub use transform::*;
pub use vertex::*;
pub use texture::*;


pub use wgpu::SurfaceError;
pub use wgpu::FilterMode;