use super::*;

#[derive(Default)]
pub struct GalleryData {
    pub current_house: usize,
    pub panel: Option<legion::Entity>,
}
