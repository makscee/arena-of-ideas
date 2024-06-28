use super::*;

#[derive(Clone, Copy)]
pub enum Icon {
    Lightning,
    Target,
    Flame,
    Discord,
    Youtube,
    Itch,
    Github,
    FFBack,
    FFForward,
    SkipBack,
    SkipForward,
    Play,
    Pause,
}

impl Icon {
    pub fn show(self, ui: &mut Ui) {
        self.image().fit_to_original_size(1.0).ui(ui);
    }

    pub fn image(self) -> Image<'static> {
        match self {
            Icon::Lightning => Image::new(include_image!("../../assets/svg/lightning.svg"))
                .fit_to_original_size(1.0),
            Icon::Target => {
                Image::new(include_image!("../../assets/svg/target.svg")).fit_to_original_size(1.0)
            }
            Icon::Flame => {
                Image::new(include_image!("../../assets/svg/flame.svg")).fit_to_original_size(1.0)
            }
            Icon::Discord => {
                Image::new(include_image!("../../assets/svg/discord.svg")).fit_to_original_size(0.8)
            }
            Icon::Youtube => {
                Image::new(include_image!("../../assets/svg/youtube.svg")).fit_to_original_size(0.8)
            }
            Icon::Itch => {
                Image::new(include_image!("../../assets/svg/itch.svg")).fit_to_original_size(0.8)
            }
            Icon::Github => {
                Image::new(include_image!("../../assets/svg/github.svg")).fit_to_original_size(0.8)
            }
            Icon::FFBack => {
                Image::new(include_image!("../../assets/svg/ff_back.svg")).fit_to_original_size(1.0)
            }
            Icon::FFForward => Image::new(include_image!("../../assets/svg/ff_forward.svg"))
                .fit_to_original_size(1.0),
            Icon::SkipBack => {
                Image::new(include_image!("../../assets/svg/skip_b.svg")).fit_to_original_size(1.0)
            }
            Icon::SkipForward => {
                Image::new(include_image!("../../assets/svg/skip_f.svg")).fit_to_original_size(1.0)
            }
            Icon::Play => {
                Image::new(include_image!("../../assets/svg/play.svg")).fit_to_original_size(1.0)
            }
            Icon::Pause => {
                Image::new(include_image!("../../assets/svg/pause.svg")).fit_to_original_size(1.0)
            }
        }
    }
}
