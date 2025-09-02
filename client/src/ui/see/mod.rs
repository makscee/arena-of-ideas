mod builder;
mod card;
mod ctxbtn;
mod info;
mod node_link_rating;
mod node_rating;
mod recursive;
mod show;
mod tag;
mod tag_card;
mod title;

use super::*;

pub use builder::*;
pub use card::*;
pub use ctxbtn::*;
pub use info::*;
pub use node_link_rating::*;
pub use node_rating::*;
pub use recursive::*;
pub use show::*;
pub use tag::*;
pub use tag_card::*;
pub use title::*;

pub trait See: Sized {
    fn see<'a>(&'a self, context: &'a Context<'a>) -> SeeBuilder<'a, Self> {
        SeeBuilder::new(self, context)
    }

    fn see_mut<'a>(&'a mut self, context: &'a Context<'a>) -> SeeBuilderMut<'a, Self> {
        SeeBuilderMut::new(self, context)
    }
}

impl<T> See for T {}
