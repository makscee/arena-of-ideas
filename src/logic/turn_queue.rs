use super::*;

const UNIT_STARTING_OFFSET: Vec2<f32> = vec2(0.5, 0.0);
const TIMER_PRE_STRIKE: f32 = 0.3;
const TIMER_POST_STRIKE: f32 = 0.05;
const TIMER_STRIKE: f32 = 0.05;
impl Logic {
    pub fn process_turn(&mut self) {
        if !self.effects.is_empty() {
            return;
        }
        if self.check_end() {
            if self.model.lives <= 0 {
                return;
            }
            if self.model.units.iter().any(|x| x.faction == Faction::Enemy) {
                self.model.lives -= 1;
            }
            self.model.transition = self.model.lives > 0;
            self.effects.clear();
            return;
        }
        debug!("Process turn phase = {:?}", self.model.phase.turn_phase);

        match self.model.phase.turn_phase {
            TurnPhase::None => {
                self.model.phase.player = self
                    .model
                    .units
                    .iter()
                    .find(|u| u.position.x == 0 && u.faction == Faction::Player)
                    .expect("Front player unit not found")
                    .id;
                self.model.phase.enemy = self
                    .model
                    .units
                    .iter()
                    .find(|u| u.position.x == 0 && u.faction == Faction::Enemy)
                    .expect("Front enemy unit not found")
                    .id;
                self.model.phase.turn_phase = TurnPhase::PreStrike;
            }
            TurnPhase::PreStrike => self.model.phase.turn_phase = TurnPhase::Strike,
            TurnPhase::Strike => self.model.phase.turn_phase = TurnPhase::PostStrike,
            TurnPhase::PostStrike => {
                self.model.phase.turn_phase = TurnPhase::None;
                self.effects
                    .add_delay_by_id("Turn".to_owned(), TurnPhase::None.duration().as_f32());
                return;
            }
        }

        let effect = Effect::Turn(Box::new(TurnEffect {
            phase: self.model.phase.turn_phase.clone(),
            player: self.model.phase.player,
            enemy: self.model.phase.enemy,
        }));
        self.effects.push_back(
            EffectContext {
                queue_id: Some("Turn".to_owned()),
                owner: 0,
                creator: 0,
                target: 0,
                vars: default(),
                status_id: None,
                color: Rgba::BLACK,
            },
            effect,
        )
    }

    fn check_end(&mut self) -> bool {
        self.model
            .units
            .iter()
            .unique_by(|unit| unit.faction)
            .count()
            < 2
    }
}
