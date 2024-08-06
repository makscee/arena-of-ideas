use chrono::Utc;

use super::*;

pub struct MetaPlugin;

impl Plugin for MetaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MetaResource>();
    }
}

#[derive(Resource, Default)]
struct MetaResource {
    state: SubState,
}

#[derive(PartialEq, Copy, Clone, EnumIter, Display, Default)]
enum SubState {
    #[default]
    Shop,
    Inventory,
}

impl MetaPlugin {
    pub fn ui_tiles(ctx: &egui::Context, world: &mut World) {
        let mut r = world.resource_mut::<MetaResource>();
        let state = SubsectionMenu::new(r.state).show(ctx);
        r.state = state;
        Tile::left("Meta").open().show(ctx, |ui| match state {
            SubState::Shop => {
                text_dots_text(
                    &"credits".cstr(),
                    &TWallet::current()
                        .amount
                        .to_string()
                        .cstr_cs(YELLOW, CstrStyle::Bold),
                    ui,
                );
                br(ui);
                let now = Utc::now().timestamp();
                let last_refresh =
                    Duration::from_micros(GlobalData::current().last_shop_refresh).as_secs() as i64;
                let period = GlobalSettings::current().meta.shop_refresh_period_secs as i64;
                let til_refresh = period - now + last_refresh;
                "Refresh in "
                    .cstr()
                    .push(
                        format!(
                            "{:02}:{:02}:{:02}",
                            til_refresh / 3600,
                            til_refresh / 60 % 60,
                            til_refresh % 60
                        )
                        .cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold),
                    )
                    .label(ui);
                TMetaShop::iter()
                    .sorted_by_key(|d| d.id)
                    .collect_vec()
                    .show_table("Meta Shop", ui, world);
            }
            SubState::Inventory => {
                TItem::iter()
                    .collect_vec()
                    .show_table("Inventory", ui, world);
            }
        });
    }
}
