use super::*;

#[derive(Default)]
pub struct Fonts {
    fonts: Vec<Rc<geng::font::Ttf>>,
    texture_cache: Vec<HashMap<String, ugli::Texture>>,
}

impl Fonts {
    pub fn new(geng: &Geng) -> Self {
        let fonts = vec![
            Rc::new(
                geng::font::Ttf::new(
                    geng,
                    include_bytes!("../../static/font/roboto.ttf"),
                    geng::font::ttf::Options {
                        pixel_size: 32.0,
                        max_distance: 0.25,
                    },
                )
                .unwrap(),
            ),
            Rc::new(
                geng::font::Ttf::new(
                    geng,
                    include_bytes!("../../static/font/zorque.ttf"),
                    geng::font::ttf::Options {
                        pixel_size: 32.0,
                        max_distance: 0.25,
                    },
                )
                .unwrap(),
            ),
            Rc::new(
                geng::font::Ttf::new(
                    geng,
                    include_bytes!("../../static/font/amontesa.ttf"),
                    geng::font::ttf::Options {
                        pixel_size: 32.0,
                        max_distance: 0.25,
                    },
                )
                .unwrap(),
            ),
        ];
        Self {
            texture_cache: Vec::from_iter((0..fonts.len()).map(|_| HashMap::default())),
            fonts,
        }
    }

    pub fn get_font(&self, index: usize) -> Rc<geng::font::Ttf> {
        self.fonts[index].clone()
    }

    pub fn load_textures(&mut self, texts: Vec<(usize, &String)>) {
        for (font, text) in texts {
            if !self.texture_cache[font].contains_key(text) {
                let texture = self.fonts[font].create_text_sdf(text, geng::TextAlign::CENTER, 64.0);
                if let Some(texture) = texture {
                    self.texture_cache[font].insert(text.clone(), texture);
                }
            }
        }
    }

    pub fn get_texture(&self, font: usize, text: &str) -> Option<&ugli::Texture> {
        return self.texture_cache[font].get(text);
    }
}
