//! # Baguette-Egui
//! 
//!  contains an integration of egui specifically for the baguette engine

pub use egui::ahash;
pub use egui;

/// handle to egui
#[derive(Default)]
pub struct Context
{
    ctx: egui::Context,
    input: egui::RawInput
}

impl Context
{
    pub fn set_input(&mut self, input: egui::RawInput)
    {
        self.input = input
    }

    pub fn run(&mut self) -> (egui::TexturesDelta, Vec<egui::ClippedPrimitive>)
    {
        let output = self.ctx.run(self.input.clone(), |_ctx| ());

        let clipped_primitives = self.ctx.tessellate(output.shapes, output.pixels_per_point);
        (output.textures_delta, clipped_primitives)
    }
}
