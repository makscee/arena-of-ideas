use super::*;

pub struct HeroGallery;

impl Plugin for HeroGallery {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::HeroGallery), Self::on_enter)
            .add_systems(OnExit(GameState::HeroGallery), Self::on_leave)
            .add_systems(Update, Self::ui.run_if(in_state(GameState::HeroGallery)));
    }
}

impl HeroGallery {
    fn on_enter(world: &mut World) {
        let team = PackedTeam::spawn(Faction::Left, world);
        let heroes = Pools::get(world).heroes.clone();
        let per_slot = vec2(3.0, 0.0);
        let start_pos = -((heroes.len() - 1) as f32) * 0.5 * per_slot + vec2(0.0, -3.0);
        heroes.into_iter().enumerate().for_each(|(slot, (_, u))| {
            let u = u.unpack(team, None, world);
            VarState::get_mut(u, world).init(
                VarName::Position,
                VarValue::Vec2(start_pos + per_slot * slot as f32),
            );
        });
        ActionPlugin::set_timeframe(1.3, world);
    }

    fn on_leave(world: &mut World) {
        UnitPlugin::despawn_all_teams(world);
        Representation::despawn_all(world);
    }

    fn ui(world: &mut World) {
        for unit in UnitPlugin::collect_faction(Faction::Left, world) {
            if let Ok(card) = UnitCard::from_entity(unit, world) {
                card.show_window(world);
            }
        }
    }
}
