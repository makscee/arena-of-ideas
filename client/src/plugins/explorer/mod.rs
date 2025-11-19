mod actions;
mod panes;
mod state;

use super::*;
pub use actions::*;
pub use panes::*;
pub use state::*;

pub struct ExplorerPlugin;

impl Plugin for ExplorerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExplorerState>();
    }
}

impl ExplorerPlugin {
    fn update_state(world: &mut World) {
        let actions = {
            let mut state = world.resource_mut::<ExplorerState>();
            std::mem::take(&mut state.pending_actions)
        };

        for action in actions {
            match action {
                ExplorerAction::InspectUnit(id) => {
                    let mut state = world.resource_mut::<ExplorerState>();
                    state.inspected_unit = Some(id);
                }
                ExplorerAction::InspectHouse(id) => {
                    let mut state = world.resource_mut::<ExplorerState>();
                    state.inspected_house = Some(id);
                }
                _ => {}
            }
        }
    }

    pub fn pane(pane: ExplorerPane, ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        Self::update_state(world);

        match &pane {
            ExplorerPane::UnitsList => ExplorerPanes::pane_units_list(ui, world)?,
            ExplorerPane::HousesList => ExplorerPanes::pane_houses_list(ui, world)?,

            ExplorerPane::UnitCard => ExplorerPanes::pane_unit_card(ui, world)?,
            ExplorerPane::HouseCard => ExplorerPanes::pane_house_card(ui, world)?,

            ExplorerPane::UnitDescription => ExplorerPanes::pane_unit_description(ui, world)?,
            ExplorerPane::UnitBehavior => ExplorerPanes::pane_unit_behavior(ui, world)?,
            ExplorerPane::UnitRepresentation => ExplorerPanes::pane_unit_representation(ui, world)?,
            ExplorerPane::UnitStats => ExplorerPanes::pane_unit_stats(ui, world)?,

            ExplorerPane::HouseColor => ExplorerPanes::pane_house_color(ui, world)?,
            ExplorerPane::AbilityMagic => ExplorerPanes::pane_ability_magic(ui, world)?,
            ExplorerPane::AbilityDescription => ExplorerPanes::pane_ability_description(ui, world)?,
            ExplorerPane::AbilityEffect => ExplorerPanes::pane_ability_effect(ui, world)?,
            ExplorerPane::StatusMagic => ExplorerPanes::pane_status_magic(ui, world)?,
            ExplorerPane::StatusDescription => ExplorerPanes::pane_status_description(ui, world)?,
            ExplorerPane::StatusBehavior => ExplorerPanes::pane_status_behavior(ui, world)?,
        }

        Ok(())
    }
}

#[derive(PartialEq, Eq, Clone, Hash, AsRefStr, Serialize, Deserialize, Debug, Display, Copy)]
pub enum ExplorerPane {
    UnitsList,
    HousesList,

    UnitCard,
    HouseCard,

    // Unit component panes
    UnitDescription,
    UnitBehavior,
    UnitRepresentation,
    UnitStats,

    // House component panes
    HouseColor,
    AbilityMagic,
    AbilityDescription,
    AbilityEffect,
    StatusMagic,
    StatusDescription,
    StatusBehavior,
}
