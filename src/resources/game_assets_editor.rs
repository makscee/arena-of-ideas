use super::*;

pub struct GameAssetsEditor;

impl GameAssetsEditor {
    pub fn open_houses_window(world: &mut World) {
        // let mut houses = houses()
        //     .clone()
        //     .into_iter()
        //     .sorted_by_key(|(name, _)| name.clone())
        //     .collect_vec();
        // Window::new("Houses Editor", move |ui, _| {
        //     if "Export"
        //         .cstr_cs(tokens_global().high_contrast_text(), CstrStyle::Bold)
        //         .button(ui)
        //         .clicked()
        //     {
        //         let path = "./assets/ron/";
        //         for (_, house) in &houses {
        //             let dir = house.to_dir("houses".into());
        //             let dir = dir.as_dir().unwrap();
        //             std::fs::create_dir_all(format!("{path}{}", dir.path().to_str().unwrap()))
        //                 .unwrap();
        //             dir.extract(path).unwrap();
        //         }
        //     }
        //     for (name, house) in &mut houses {
        //         CollapsingHeader::new(&*name).show(ui, |ui| {
        //             house.show_mut(None, ui);
        //         });
        //     }
        // })
        // .push(world);
    }
}
