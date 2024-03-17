use bevy_egui::egui::DragValue;

use super::*;

pub struct HeroGallery;

impl Plugin for HeroGallery {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::HeroGallery), Self::reload)
            .add_systems(OnExit(GameState::HeroGallery), Self::on_leave)
            .add_systems(Update, Self::ui.run_if(in_state(GameState::HeroGallery)));
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HeroGalleryData {
    per_row: usize,
    show_limit: usize,
    offset: Vec2,
    cards: bool,
}

impl Default for HeroGalleryData {
    fn default() -> Self {
        Self {
            per_row: 10,
            offset: vec2(3.0, -3.0),
            show_limit: default(),
            cards: default(),
        }
    }
}

impl HeroGallery {
    fn reload(world: &mut World) {
        let data = PersistentData::load(world).hero_gallery_data;
        let team = TeamPlugin::spawn(Faction::Left, world);
        let heroes = Pools::get(world)
            .heroes
            .values()
            .cloned()
            .sorted_by_key(|v| v.houses.clone())
            .collect_vec();
        let columns = data.per_row.max(1).min(heroes.len()) as f32 - 1.0;
        let rows = (heroes.len() as f32 / data.per_row as f32).ceil() - 1.0;
        let start_pos = vec2(-columns * 0.5 * data.offset.x, -rows * 0.5 * data.offset.y);

        let mut row = 0;
        let mut col = 0;
        for u in heroes {
            let u = u.unpack(team, None, world);
            let pos = start_pos + data.offset * vec2(col as f32, row as f32);
            VarState::get_mut(u, world).init(VarName::Position, VarValue::Vec2(pos));
            col += 1;
            if data.per_row > 0 && data.per_row <= col {
                col = 0;
                row += 1;
            }
        }
    }

    fn ui(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };
        let mut pd = PersistentData::load(world);
        let mut data = pd.hero_gallery_data.clone();
        TopBottomPanel::bottom("gallery controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("per row:");
                DragValue::new(&mut data.per_row).ui(ui);
            });
        });
        if !data.eq(&pd.hero_gallery_data) {
            pd.hero_gallery_data = data;
            pd.save(world).unwrap();
            Self::reload(world);
        }
    }

    fn on_leave(world: &mut World) {
        UnitPlugin::despawn_all_teams(world);
        Representation::despawn_all(world);
    }
}
