mod builder;
mod composers;
mod features;
mod recursive_impl;

pub use builder::*;
pub use composers::*;
pub use features::*;

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
