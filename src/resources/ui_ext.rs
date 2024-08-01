use super::*;

pub trait ShowTable<T> {
    fn show_table(&self, name: &'static str, ui: &mut Ui, world: &mut World) -> TableState {
        self.show_modified_table(name, ui, world, |t| t)
    }
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: fn(Table<T>) -> Table<T>,
    ) -> TableState;
}

impl ShowTable<TTeam> for Vec<TTeam> {
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: fn(Table<TTeam>) -> Table<TTeam>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .selectable()
            .column_cstr("units", |d: &TTeam| d.cstr());
        t = m(t);
        t.ui(self, ui, world)
    }
}
impl ShowTable<TBaseUnit> for Vec<TBaseUnit> {
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: fn(Table<TBaseUnit>) -> Table<TBaseUnit>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .column_cstr("name", |d: &TBaseUnit| d.name.cstr_c(name_color(&d.name)))
            .column_cstr("house", |d| {
                let color = name_color(&d.house);
                d.house.cstr_c(color)
            })
            .column_int("pwr", |d| d.pwr)
            .column_int("hp", |d| d.hp)
            .column(
                "rarity",
                |u| (u.rarity as i32).into(),
                |u, _, ui, _| Rarity::from_int(u.rarity).cstr().label(ui),
            );
        t = m(t);
        t.ui(self, ui, world)
    }
}
impl ShowTable<FusedUnit> for Vec<FusedUnit> {
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: fn(Table<FusedUnit>) -> Table<FusedUnit>,
    ) -> TableState {
        let mut t = Table::new(name).title().column(
            "name",
            |d: &FusedUnit| d.id.into(),
            |d, _, ui, _| {
                let r = d.cstr_limit(0).button(ui);
                if r.clicked() {
                    Tile::add_fused_unit(d.clone(), ui.ctx());
                }
                r
            },
        );
        t = m(t);
        t.ui(self, ui, world)
    }
}
impl ShowTable<TArenaLeaderboard> for Vec<TArenaLeaderboard> {
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: fn(Table<TArenaLeaderboard>) -> Table<TArenaLeaderboard>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .column_int("round", |d: &TArenaLeaderboard| d.round as i32)
            .column_int("score", |d| d.score as i32)
            .column_ts("time", |d| d.ts)
            .column_cstr("team", |d| {
                d.team.get_team().cstr().style(CstrStyle::Small).take()
            })
            .column_user_click(
                "owner",
                |d| d.user,
                |gid, ui, _| Tile::add_user(gid, ui.ctx()),
            )
            .column_cstr("mode", |d| {
                match &d.mode {
                    GameMode::ArenaNormal => "normal".into(),
                    GameMode::ArenaConst(seed) => format!("const {seed}"),
                }
                .cstr_cs(VISIBLE_DARK, CstrStyle::Small)
            });
        t = m(t);
        t.ui(self, ui, world)
    }
}

pub trait Show {
    fn show(&self, ui: &mut Ui, world: &mut World);
}

impl Show for TUser {
    fn show(&self, ui: &mut Ui, _: &mut World) {
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
        title("Fused Unit", ui);
        text_dots_text(
            &"gid".cstr(),
            &self.id.to_string().cstr_c(VISIBLE_LIGHT),
            ui,
        );
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
        text_dots_text(
            &"gid".cstr(),
            &self.id.to_string().cstr_c(VISIBLE_LIGHT),
            ui,
        );
        ui.push_id(self.id, |ui| {
            self.units.show_table("Units", ui, world);
        });
    }
}
