use super::*;

pub struct BattleTestingPlugin;

struct BattleTestingData {
    world: World,
    battle: Battle,
}

impl BattleTestingPlugin {
    pub fn pane(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        Ok(())
    }
}
