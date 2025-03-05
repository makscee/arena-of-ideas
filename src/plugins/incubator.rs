use super::*;

pub struct IncubatorPlugin;

impl Plugin for IncubatorPlugin {
    fn build(&self, app: &mut App) {}
}

impl IncubatorPlugin {
    pub fn tab_units(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let units = All::get_by_id(0, world)
            .unwrap()
            .incubator_load(world)?
            .units_load(world);
        ui.vertical(|ui| {
            for unit in units {
                unit.name.label(ui);
            }
        });
        Ok(())
    }
}
