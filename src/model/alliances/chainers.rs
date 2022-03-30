use super::*;

pub fn initialize(logic: &mut Logic, party_members: usize) {
    for template in logic.model.unit_templates.values_mut() {
        if !template.alliances.contains(&Alliance::Chainers) {
            continue;
        }
        template.walk_effects_mut(&mut |effect| match effect {
            Effect::Chain(chain) => {
                if party_members >= 2 {
                    chain.jump_distance *= r32(2.0);
                }
                if party_members >= 4 {
                    chain.targets += 2;
                }
                if party_members >= 6 {
                    chain.split_probability = min(chain.split_probability + r32(0.1), r32(1.0));
                }
            }
            _ => {}
        });
    }
}
