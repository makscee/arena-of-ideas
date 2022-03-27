use super::*;

pub fn initialize(logic: &mut Logic, party_members: usize) {
    for template in logic.model.unit_templates.values_mut() {
        if !template.alliances.contains(&Alliance::Warriors) {
            continue;
        }
        let mut protection = 0.0;
        if party_members >= 3 {
            protection = 30.0;
        } else if party_members >= 6 {
            protection = 50.0;
        }
        if protection != 0.0 {
            template.statuses.push(AttachedStatus {
                status: Status::Protection(Box::new(ProtectionStatus {
                    percent: protection,
                })),
                time: None,
            });
        }
    }
}
