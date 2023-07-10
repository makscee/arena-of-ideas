

#[derive(Default)]
pub struct GalleryData {
    pub current_house: usize,
    pub panel: Option<legion::Entity>,
    pub card: bool,
    pub cur_card: f32,
}
