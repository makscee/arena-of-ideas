use super::*;

pub fn initialize(logic: &mut Logic, party_members: usize) {
    for template in logic.model.unit_templates.values_mut() {
        if !template.alliances.contains(&Alliance::Exploders) {
            continue;
        }
        template.walk_effects_mut(&mut |effect| {
            if let Effect::AOE(aoe) = effect {
                if party_members >= 2 {
                    aoe.radius *= Coord::new(1.4);
                }
                if party_members >= 4 {
                    // Seems like Explode needs to be a separate effect to count the damage done
                    aoe.effect = Effect::List(Box::new(ListEffect {
                        effects: vec![aoe.effect.clone(), todo!()],
                    }));
                }
                if party_members >= 6 {
                    *effect = Effect::Random(Box::new(RandomEffect {
                        choices: vec![
                            WeightedEffect {
                                weight: 80.0,
                                effect: effect.clone(),
                            },
                            WeightedEffect {
                                weight: 20.0,
                                effect: Effect::List(Box::new(ListEffect {
                                    effects: vec![effect.clone(), effect.clone()],
                                })),
                            },
                        ],
                    }));
                }
            }
        });
    }
}
