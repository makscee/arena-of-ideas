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
            ExplorerPane::AbilitiesList => ExplorerPanes::pane_abilities_list(ui, world)?,
            ExplorerPane::StatusesList => ExplorerPanes::pane_statuses_list(ui, world)?,

            ExplorerPane::HouseAbilitiesList => {
                ExplorerPanes::pane_house_abilities_list(ui, world)?
            }
            ExplorerPane::HouseStatusesList => ExplorerPanes::pane_house_statuses_list(ui, world)?,

            ExplorerPane::UnitParentList => ExplorerPanes::pane_unit_parent_list(ui, world)?,
            ExplorerPane::AbilityParentList => ExplorerPanes::pane_ability_parent_list(ui, world)?,
            ExplorerPane::StatusParentList => ExplorerPanes::pane_status_parent_list(ui, world)?,

            ExplorerPane::UnitCard => ExplorerPanes::pane_unit_card(ui, world)?,
            ExplorerPane::HouseCard => ExplorerPanes::pane_house_card(ui, world)?,

            ExplorerPane::UnitDescription => ExplorerPanes::pane_unit_description(ui, world)?,
            ExplorerPane::UnitBehavior => ExplorerPanes::pane_unit_behavior(ui, world)?,
            ExplorerPane::UnitStats => ExplorerPanes::pane_unit_stats(ui, world)?,
            ExplorerPane::UnitRepresentation => ExplorerPanes::pane_unit_representation(ui, world)?,

            ExplorerPane::HouseColor => ExplorerPanes::pane_house_color(ui, world)?,
        }

        Ok(())
    }
}

#[derive(PartialEq, Eq, Clone, Hash, AsRefStr, Serialize, Deserialize, Debug, Display, Copy)]
pub enum ExplorerPane {
    UnitsList,
    HousesList,
    AbilitiesList,
    StatusesList,

    HouseAbilitiesList,
    HouseStatusesList,

    UnitParentList,
    AbilityParentList,
    StatusParentList,

    UnitCard,
    HouseCard,
    UnitDescription,
    UnitBehavior,
    UnitStats,
    UnitRepresentation,

    HouseColor,
}
