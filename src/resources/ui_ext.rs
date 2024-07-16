use super::*;

pub trait ShowTable {
    fn show_table(&self, name: &'static str, ui: &mut Ui, world: &mut World) -> TableState;
}

impl ShowTable for Vec<TTeam> {
    fn show_table(&self, name: &'static str, ui: &mut Ui, world: &mut World) -> TableState {
        Table::new(name)
            .title()
            .selectable()
            .column_cstr("units", |d: &TTeam| d.cstr())
            .ui(self, ui, world)
    }
}
impl ShowTable for Vec<TBaseUnit> {
    fn show_table(&self, name: &'static str, ui: &mut Ui, world: &mut World) -> TableState {
        Table::new(name)
            .title()
            .column_cstr("name", |d: &TBaseUnit| d.name.cstr_c(name_color(&d.name)))
            .column_int("pwr", |d| d.pwr)
            .column_int("hp", |d| d.hp)
            .ui(self, ui, world)
    }
}
impl ShowTable for Vec<FusedUnit> {
    fn show_table(&self, name: &'static str, ui: &mut Ui, world: &mut World) -> TableState {
        Table::new(name)
            .title()
            .selectable()
            .column_cstr("name", |u: &FusedUnit| u.cstr())
            .ui(self, ui, world)
    }
}

pub trait Show {
    fn show(&self, ui: &mut Ui, world: &mut World);
}

impl Show for TUser {
    fn show(&self, ui: &mut Ui, world: &mut World) {
        text_dots_text(
            &"name".cstr(),
            &self.name.cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold),
            ui,
        );
        text_dots_text(&"id".cstr(), &self.id.to_string().cstr_c(VISIBLE_LIGHT), ui);
    }
}
impl Show for FusedUnit {
    fn show(&self, ui: &mut Ui, world: &mut World) {
        self.bases
            .iter()
            .filter_map(|b| TBaseUnit::filter_by_name(b.clone()))
            .collect_vec()
            .show_table("Bases", ui, world);
    }
}
impl Show for TTeam {
    fn show(&self, ui: &mut Ui, world: &mut World) {
        title("Team", ui);
        text_dots_text(&"owner".cstr(), &self.owner.get_user().cstr(), ui);
        ui.push_id(self.id, |ui| {
            let state = self.units.show_table("Units", ui, world);
            if let Some(selected) = state.selected_row {
                let unit = &self.units[selected];
                ui.push_id("fuse bases", |ui| {
                    unit.show(ui, world);
                });
            }
        });
    }
}
