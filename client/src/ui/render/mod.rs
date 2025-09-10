mod builder;
mod composers;
mod composers_advanced;
mod features;
mod features_impl;
mod integration;
mod test;

pub use builder::*;
pub use composers::*;
pub use composers_advanced::*;
pub use features::*;
pub use integration::*;
pub use test::*;

use super::*;

/// Main trait that types implement to get access to the render builder
pub trait Render: Sized {
    fn render<'a>(&'a self, context: &'a Context<'a>) -> RenderBuilder<'a, Self> {
        RenderBuilder::new(self, context)
    }

    fn render_mut<'a>(&'a mut self, context: &'a Context<'a>) -> RenderBuilder<'a, Self> {
        RenderBuilder::new_mut(self, context)
    }
}

/// Blanket implementation for all types
impl<T> Render for T {}
