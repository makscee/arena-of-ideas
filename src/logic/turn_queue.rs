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
        if !self.model.in_battle {
            self.model.transition = self.model.lives > 0;
            return;
        }
        if let Some(victory) = self.check_end() {
            if self.model.lives <= 0 {
                return;
            }
            let effect = if victory {
                self.sound_controller.win();
                Panel::create(
                    "Victory".to_owned(),
                    r32(2.0),
                    Some(Rgba::try_from("#23ff40").unwrap()),
                )
            } else {
                self.model.lives -= 1;
                self.sound_controller.lose();
                Panel::create(
                    "Defeat".to_owned(),
                    r32(2.0),
                    Some(Rgba::try_from("#7c0000").unwrap()),
                )
            };
            self.effects.push_front(EffectContext::empty(), effect);
            self.model.in_battle = false;
            return;
        }
        debug!("Process turn phase = {:?}", self.model.phase.turn_phase);

        match self.model.phase.turn_phase {
            TurnPhase::None => {
                self.model.phase.player = self
                    .model
                    .units
                    .iter()
                    .filter(|u| u.faction == Faction::Player)
                    .min_by(|x, y| x.position.x.cmp(&y.position.x))
                    .expect("Front player unit not found")
                    .id;
                self.model.phase.enemy = self
                    .model
                    .units
                    .iter()
                    .filter(|u| u.faction == Faction::Enemy)
                    .min_by(|x, y| x.position.x.cmp(&y.position.x))
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

    pub fn check_end(&mut self) -> Option<bool> {
        // true = victory, false = lose
        if self
            .model
            .units
            .iter()
            .unique_by(|unit| unit.faction)
            .count()
            < 2
        {
            Some(!self.model.units.iter().any(|x| x.faction == Faction::Enemy))
        } else {
            None
        }
    }
}
