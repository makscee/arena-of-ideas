mod actions;
mod panes;
mod state;

use super::*;
pub use actions::*;
pub use panes::*;
pub use state::*;

pub struct IncubatorPlugin;

impl Plugin for IncubatorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IncubatorState>();
    }
}

impl IncubatorPlugin {
    fn update_state(world: &mut World) {
        let actions = {
            let mut state = world.resource_mut::<IncubatorState>();
            std::mem::take(&mut state.pending_actions)
        };

        for action in actions {
            match action {
                IncubatorAction::InspectUnit(id) => {
                    let mut state = world.resource_mut::<IncubatorState>();
                    state.inspected_unit = Some(id);
                }
                IncubatorAction::InspectHouse(id) => {
                    let mut state = world.resource_mut::<IncubatorState>();
                    state.inspected_house = Some(id);
                }
                _ => {}
            }
        }
    }

    pub fn pane(pane: IncubatorPane, ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        Self::update_state(world);

        match &pane {
            IncubatorPane::UnitsList => IncubatorPanes::pane_units_list(ui, world)?,
            IncubatorPane::HousesList => IncubatorPanes::pane_houses_list(ui, world)?,

            IncubatorPane::UnitCard => IncubatorPanes::pane_unit_card(ui, world)?,
            IncubatorPane::HouseCard => IncubatorPanes::pane_house_card(ui, world)?,

            IncubatorPane::UnitDescription => IncubatorPanes::pane_unit_description(ui, world)?,
            IncubatorPane::UnitBehavior => IncubatorPanes::pane_unit_behavior(ui, world)?,
            IncubatorPane::UnitRepresentation => {
                IncubatorPanes::pane_unit_representation(ui, world)?
            }
            IncubatorPane::UnitStats => IncubatorPanes::pane_unit_stats(ui, world)?,

            IncubatorPane::HouseColor => IncubatorPanes::pane_house_color(ui, world)?,
            IncubatorPane::AbilityMagic => IncubatorPanes::pane_ability_magic(ui, world)?,
            IncubatorPane::AbilityDescription => {
                IncubatorPanes::pane_ability_description(ui, world)?
            }
            IncubatorPane::AbilityEffect => IncubatorPanes::pane_ability_effect(ui, world)?,
            IncubatorPane::StatusMagic => IncubatorPanes::pane_status_magic(ui, world)?,
            IncubatorPane::StatusDescription => IncubatorPanes::pane_status_description(ui, world)?,
            IncubatorPane::StatusBehavior => IncubatorPanes::pane_status_behavior(ui, world)?,
        }

        Ok(())
    }
}

#[derive(PartialEq, Eq, Clone, Hash, AsRefStr, Serialize, Deserialize, Debug, Display, Copy)]
pub enum IncubatorPane {
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
