pub use input::winit::window::Window;
pub use input::*;
pub use baguette_math::*;
/// image crate reexport
pub use image;

#[path ="rendering/renderer.rs"]
pub mod renderer;
#[path ="rendering/camera.rs"]
pub mod camera;
#[path ="rendering/renderpasses.rs"]
mod renderpasses;
pub use renderpasses::*;

pub use spritepass::*;
pub use resolutionpass::*;

#[path ="rendering/postprocess/postprocess.rs"]
pub mod postprocess;
pub use postprocess::*;

#[path ="rendering/postprocess/noise.rs"]
pub mod noise;
#[path ="rendering/postprocess/blur.rs"]
pub mod blur;

#[path ="rendering/raw/transform.rs"]
pub mod transform;
#[path ="rendering/raw/vertex.rs"]
pub mod vertex;
#[path ="rendering/raw/texture.rs"]
pub mod texture;
#[path ="rendering/raw/mesh.rs"]
pub mod mesh;

#[path ="rendering/renderer/util.rs"]
pub mod util;

pub use transform::*;
pub use vertex::*;
pub use texture::*;
pub use mesh::*;
pub use util::*;

pub use renderer::*;
pub use camera::*;

pub(crate) use renderer::static_render_data::*;
//pub(crate) use baguette_proc::*;
//pub(crate) use gameloop::Update;
pub use wgpu::SurfaceError;