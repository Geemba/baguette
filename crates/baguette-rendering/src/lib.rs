//! rendering module of baguette

pub use input::winit::window::Window;

pub(crate) use baguette_math::*;
pub(crate) use sprite::{SPRITE_INDICES_U16, SPRITE_INDICES_U32};

/// image crate reexport
pub use image;

#[path ="rendering/2d/sprite.rs"]
pub mod sprite;
pub use sprite::Sprite;
pub use sprite::SpriteLayout;
pub use sprite::SpriteInstance;

#[path ="rendering/2d/spritesheet.rs"]
pub(crate) mod spritesheet;
pub use spritesheet::SheetSlices;
pub use spritesheet::SpriteSheet;
pub use spritesheet::SpriteSheetBuilder;

#[path = "rendering/2d/spritepass.rs"]
pub(crate) mod spritepass;
pub use spritepass::*;

#[path = "rendering/2d/tilemap.rs"]
pub(crate) mod tilemap;
pub use tilemap::*;

#[path ="rendering/renderer.rs"]
pub mod renderer;
pub use renderer::*;

#[path ="rendering/ui/ui.rs"]
pub mod ui;
//pub use ui::*;

#[path ="rendering/renderpasses.rs"]
mod renderpasses;
pub use renderpasses::*;

#[path ="rendering/camera.rs"]
pub mod camera;
pub use camera::*;

#[path ="rendering/raw/texture.rs"]
pub mod texture;

#[path ="rendering/renderer/util.rs"]
pub mod util;
pub use texture::*;


pub use wgpu::SurfaceError;
pub use wgpu::FilterMode;