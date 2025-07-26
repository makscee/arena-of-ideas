mod builder;
mod card;
mod ctxbtn;
mod info;
mod node_link_rating;
mod node_rating;
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
pub use tag::*;
pub use tag_card::*;
pub use title::*;

pub trait See: Sized {
    fn see<'a>(&'a self, context: &'a Context<'a>) -> SeeBuilder<'a, Self> {
        SeeBuilder::new(self, context)
    }
}

impl<T> See for T {}
