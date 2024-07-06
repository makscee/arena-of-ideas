use super::*;

pub fn unit_card(t: f32, state: &VarState, ui: &mut Ui, world: &World) {
    let houses = state
        .get_value_at(VarName::Houses, t)
        .unwrap()
        .get_string_list()
        .unwrap();
    state
        .get_string_at(VarName::Name, t)
        .unwrap()
        .cstr_cs(GameAssets::color(&houses[0], world), CstrStyle::Heading)
        .label(ui);
    Middle3::default().width(70.0).ui(
        ui,
        world,
        |ui, _| {
            ui.horizontal_centered(|ui| {
                Icon::Sword.image().ui(ui);
                state
                    .get_int_at(VarName::Pwr, t)
                    .unwrap()
                    .to_string()
                    .cstr_c(YELLOW)
                    .label(ui);
            });
        },
        |ui, _| {
            ui.horizontal_centered(|ui| {
                state
                    .get_int_at(VarName::Pwr, t)
                    .unwrap()
                    .to_string()
                    .cstr_c(YELLOW)
                    .label(ui);
                Icon::Sword.image().ui(ui);
            });
        },
        |ui, _| {
            ui.horizontal_centered(|ui| {
                Icon::Sword.image().ui(ui);
                state
                    .get_int_at(VarName::Pwr, t)
                    .unwrap()
                    .to_string()
                    .cstr_c(YELLOW)
                    .label(ui);
            });
        },
    );
    let mut houses_cstr = Cstr::default();
    for house in ["Holy", "Mages", "Holy"] {
        houses_cstr.push(house.cstr_c(GameAssets::color(&house, world)));
    }
    houses_cstr.join(&" + ".cstr()).label(ui);
}
