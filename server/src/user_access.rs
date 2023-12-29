use super::*;

#[spacetimedb(table)]
pub struct UserAccess {
    #[primarykey]
    identity: Identity,
    rights: Vec<UserRight>,
}

#[derive(SpacetimeType, Debug, Eq, PartialEq)]
pub enum UserRight {
    UnitSync,
}

#[spacetimedb(reducer)]
fn give_right(ctx: ReducerContext, identity: Identity, right: UserRight) -> Result<(), String> {
    if let Some(mut access) = UserAccess::filter_by_identity(&identity) {
        if !access.rights.contains(&right) {
            access.rights.push(right);
            UserAccess::update_by_identity(&identity, access);
        }
    } else {
        UserAccess::insert(UserAccess {
            identity,
            rights: [right].into(),
        })?;
    };
    Ok(())
}

impl UserRight {
    pub fn check(self, identity: &Identity) -> Result<(), String> {
        if UserAccess::filter_by_identity(identity).is_some_and(|v| v.rights.contains(&self)) {
            Ok(())
        } else {
            Err(format!("No right {self:?}"))
        }
    }
}
