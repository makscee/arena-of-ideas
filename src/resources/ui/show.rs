use super::*;

pub trait Show {
    fn show(&self, fade_in: f32, ui: &mut Ui, world: &mut World);
}
fn show_counter(c: u32, fade_in: f32, ui: &mut Ui) {
    format!("x{}", c)
        .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold)
        .label_alpha(fade_in, ui);
}

impl Show for TPlayer {
    fn show(&self, _: f32, ui: &mut Ui, _: &mut World) {
        text_dots_text(
            "name".cstr(),
            self.name.cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold),
            ui,
        );
        text_dots_text("id".cstr(), self.id.to_string().cstr_c(VISIBLE_LIGHT), ui);
    }
}
impl Show for FusedUnit {
    fn show(&self, fade_in: f32, ui: &mut Ui, world: &mut World) {
        let r = self
            .cstr_limit(3, true)
            .as_label_alpha(fade_in, ui)
            .sense(Sense::hover())
            .ui(ui);
        if r.hovered() {
            cursor_window(ui.ctx(), |ui| match cached_fused_card(self, ui, world) {
                Ok(_) => {}
                Err(e) => error!("{e}"),
            });
        }
    }
}
impl Show for TTeam {
    fn show(&self, _: f32, ui: &mut Ui, world: &mut World) {
        title("Team", ui);
        if !self.name.is_empty() {
            text_dots_text("name".cstr(), self.name.cstr_c(VISIBLE_LIGHT), ui);
        }
        text_dots_text("owner".cstr(), self.owner.get_player().cstr(), ui);
        text_dots_text("gid".cstr(), self.id.to_string().cstr_c(VISIBLE_LIGHT), ui);
        ui.push_id(self.id, |ui| {
            Table::new_persistant("Units", self.units.clone(), world)
                .add_fused_unit_columns(|d| d)
                .ui(ui, world);
        });
    }
}
impl Show for ItemBundle {
    fn show(&self, fade_in: f32, ui: &mut Ui, world: &mut World) {
        let lines = self.units.len()
            + self.unit_shards.len()
            + self.lootboxes.len()
            + (self.credits != 0) as usize;
        let per_row = 1.0 / lines as f32;
        let mut i = 0.0;
        let mut t = move || {
            let t = smoothstep(per_row * i, per_row * (i + 1.0), fade_in);
            i += 1.0;
            t
        };
        if !self.unit_shards.is_empty() {
            br(ui);
            for id in &self.unit_shards {
                let item = id.unit_shard_item();
                item.show(t(), ui, world);
            }
        }
        if !self.units.is_empty() {
            br(ui);
            for id in &self.units {
                let item = id.unit_item();
                item.show(t(), ui, world);
            }
        }
        if !self.lootboxes.is_empty() {
            br(ui);
            for id in &self.lootboxes {
                let item = id.lootbox_item();
                item.show(t(), ui, world);
            }
        }
    }
}
impl Show for TUnitShardItem {
    fn show(&self, fade_in: f32, ui: &mut Ui, _: &mut World) {
        ui.columns(3, |ui| {
            ui[0].vertical_centered_justified(|ui| {
                self.unit
                    .cstr_cs(name_color(&self.unit), CstrStyle::Bold)
                    .label_alpha(smoothstep(0.0, 0.3, fade_in), ui);
            });
            ui[1].vertical_centered_justified(|ui| {
                let rarity = Rarity::from_base(&self.unit);
                format!("{} shard", rarity)
                    .cstr_cs(rarity.color(), CstrStyle::Bold)
                    .label_alpha(smoothstep(0.3, 0.6, fade_in), ui);
            });
            ui[2].vertical_centered_justified(|ui| {
                show_counter(self.count, smoothstep(0.6, 1.0, fade_in), ui);
            });
        });
    }
}
impl Show for TUnitItem {
    fn show(&self, fade_in: f32, ui: &mut Ui, world: &mut World) {
        ui.columns(3, |ui| {
            let base = self.unit.base_unit();
            let rarity = Rarity::from(base.rarity);
            ui[0].vertical_centered_justified(|ui| {
                self.unit.show(smoothstep(0.0, 0.3, fade_in), ui, world);
            });
            ui[1].vertical_centered_justified(|ui| {
                format!("{} unit", rarity)
                    .cstr_cs(rarity.color(), CstrStyle::Bold)
                    .label_alpha(smoothstep(0.3, 0.7, fade_in), ui);
            });
        })
    }
}
impl Show for TLootboxItem {
    fn show(&self, fade_in: f32, ui: &mut Ui, world: &mut World) {
        ui.columns(3, |ui| {
            ui[0].vertical_centered_justified(|ui| {
                "Lootbox"
                    .cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold)
                    .label_alpha(smoothstep(0.0, 0.3, fade_in), ui)
            });
            ui[1].vertical_centered_justified(|ui| {
                self.kind.show(smoothstep(0.3, 0.6, fade_in), ui, world);
            });
            ui[2].vertical_centered_justified(|ui| {
                show_counter(self.count, smoothstep(0.6, 1.0, fade_in), ui);
            });
        });
    }
}
impl Show for LootboxKind {
    fn show(&self, fade_in: f32, ui: &mut Ui, _: &mut World) {
        self.cstr().label_alpha(fade_in, ui);
    }
}
