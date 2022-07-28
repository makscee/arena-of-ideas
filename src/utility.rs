use geng::ui::Config;

use crate::Assets;
use std::collections::HashMap;
use std::collections::VecDeque;

use super::*;

pub fn rename_units(geng: &Geng, path: &std::path::Path, assets: Assets) {
    let units = path.join("units.json").to_owned();
    debug!("Start renaming");
    let mut future = async move {
        let json = std::fs::read_to_string(&units).expect("Failed to load unit packs");
        let packs: Vec<String> = serde_json::from_str(&json).expect("Failed to parse load packs");
        debug!("Updating packs: {:?}", packs);
        for pack in packs {
            let base_path = path.join(&pack);

            //Skip all but player units
            if base_path.file_name().unwrap().to_str().unwrap() != "player" {
                continue;
            }

            debug!("Updating pack {:?}", base_path);
            let list = base_path.join("_list.json");
            let json = std::fs::read_to_string(&list).expect("Failed to load pack");
            let types: Vec<String> = serde_json::from_str(&json).expect("Failed to parse pack");
            let mut _list = types.clone();
            for (i, typ) in types.into_iter().enumerate() {
                if !assets.units.map.contains_key(&typ) {
                    debug!("Unit not loaded: {:?}", &typ);
                }
                let unit = &assets.units.map[&typ];
                let old_name = &typ;
                let new_name = if unit.base.is_some() {
                    let mut base = unit.clone().base.unwrap();
                    if base.chars().last().unwrap() == '+' {
                        base.pop();
                    }
                    format!(
                        "{}_{}_{}_{}",
                        base_path
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_owned()
                            .chars()
                            .next()
                            .unwrap(),
                        unit.tier,
                        base.chars().next().unwrap(),
                        unit.name
                    )
                } else {
                    format!(
                        "{}_{}_{}",
                        base_path
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_owned()
                            .chars()
                            .next()
                            .unwrap(),
                        unit.tier,
                        unit.name
                    )
                };

                let old_file = base_path.join(format!("{}.json", old_name));
                let new_file = base_path.join(format!("{}.json", new_name));
                std::fs::rename(&old_file, &new_file).expect(&format!(
                    "Cannot rename unit asset: {:?} to {:?}",
                    old_file, new_file
                ));
                _list[i] = new_name.clone();
                let data = serde_json::to_string_pretty(&_list).expect("Failed to serialize item");
                std::fs::write(&list, data).expect(&format!("Cannot save _list: {:?}", list));
                debug!("Renaming {:?} to {:?}", old_name, new_name);
            }
            debug!("Saving pack: {:?}", base_path);
        }
    }
    .boxed_local();
    if let std::task::Poll::Ready(assets) = future.as_mut().poll(
        &mut std::task::Context::from_waker(futures::task::noop_waker_ref()),
    ) {
        debug!("Complete");
    }
}
