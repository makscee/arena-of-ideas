use super::*;

pub struct BattlePlugin;

#[derive(Resource)]
struct BattleData {
    battle: Battle,
    simulation: BattleSimulation,
    t: f32,
    playing: bool,
}

fn rm(world: &mut World) -> Mut<BattleData> {
    world.resource_mut::<BattleData>()
}

impl BattlePlugin {
    pub fn load_empty(world: &mut World) {
        let team = Team::default();
        let battle = Battle {
            left: team.clone(),
            right: team.clone(),
        };
        world.insert_resource(BattleData {
            simulation: BattleSimulation::new(battle.clone()),
            battle,
            t: 0.0,
            playing: false,
        });
    }
    fn reload_simulation(world: &mut World) {
        let mut data = rm(world);
        data.simulation = BattleSimulation::new(data.battle.clone()).start();
    }
    pub fn add_panes() {
        TilePlugin::add_to_current(|tree| {
            if let Some(id) = tree.tiles.find_pane(&Pane::Battle(BattlePane::View)) {
                tree.tiles.remove(id);
            }
            if let Some(id) = tree.tiles.find_pane(&Pane::Battle(BattlePane::Controls)) {
                tree.tiles.remove(id);
            }
            let view = tree.tiles.insert_pane(Pane::Battle(BattlePane::View));
            let controls = tree.tiles.insert_pane(Pane::Battle(BattlePane::Controls));
            tree.tiles.insert_vertical_tile([view, controls].into())
        });
    }
    pub fn pane_view(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let Some(mut data) = world.remove_resource::<BattleData>() else {
            "[red No battle loaded]".cstr().label(ui);
            return Ok(());
        };

        let t = data.t;
        data.simulation.show_at(t, ui);

        world.insert_resource(data);
        Ok(())
    }
    fn open_team_editor(left: bool, team: Team, world: &mut World) {
        TeamEditorPlugin::load_team(team, world);
        TeamEditorPlugin::on_save_fn(
            move |team, world| {
                if left {
                    rm(world).battle.left = team;
                } else {
                    rm(world).battle.right = team;
                }
                Self::reload_simulation(world);
                TilePlugin::close_match(|p| matches!(p, Pane::Team(..)));
            },
            world,
        )
        .notify(world);
        TeamEditorPlugin::unit_add_from_core(world).notify(world);
        TeamEditorPlugin::add_panes();
    }
    pub fn pane_controls(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let Some(mut data) = world.remove_resource::<BattleData>() else {
            "[red No battle loaded]".cstr().label(ui);
            return Ok(());
        };
        if "edit left team".cstr().button(ui).clicked() {
            Self::open_team_editor(true, data.battle.left.clone(), world);
        }
        if "edit right team".cstr().button(ui).clicked() {
            Self::open_team_editor(false, data.battle.right.clone(), world);
        }
        let BattleData {
            battle: _,
            simulation,
            t,
            playing,
        } = &mut data;
        if simulation.duration > 0.0 {
            Slider::new("ts")
                .full_width()
                .ui(t, 0.0..=simulation.duration, ui);
        }
        ui.horizontal(|ui| {
            Checkbox::new(playing, "play").ui(ui);
            if "+1".cstr().button(ui).clicked() {
                simulation.run();
            }
            if "+10".cstr().button(ui).clicked() {
                for _ in 0..10 {
                    simulation.run();
                }
            }
            if "+100".cstr().button(ui).clicked() {
                for _ in 0..100 {
                    simulation.run();
                }
            }
        });
        if *playing {
            *t += gt().last_delta();
            *t = t.at_most(simulation.duration);
        }
        if *t >= simulation.duration && !simulation.ended() {
            simulation.run();
        }
        world.insert_resource(data);
        Ok(())
    }
}
