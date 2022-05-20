use super::*;

pub fn initialize(logic: &mut Logic, party_members: usize) {
    for template in logic.model.unit_templates.values_mut() {
        if !template.clans.contains(&Clan::Archers) {
            continue;
        }
        template.walk_effects_mut(&mut |effect| match effect {
            Effect::Projectile(projectile) => {
                *effect = if party_members >= 6 {
                    Effect::AddTargets(Box::new(AddTargetsEffect {
                        effect: effect.clone(),
                        condition: Condition::Always,
                        additional_targets: Some(3),
                    }))
                } else if party_members >= 3 {
                    Effect::AddTargets(Box::new(AddTargetsEffect {
                        effect: effect.clone(),
                        condition: Condition::Always,
                        additional_targets: Some(1),
                    }))
                } else {
                    effect.clone()
                };
            }
            _ => {}
        });
    }
}
