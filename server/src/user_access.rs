use super::*;

#[spacetimedb(table)]
pub struct UserAccess {
    #[primarykey]
    pub identity: Identity,
    pub rights: Vec<UserRight>,
}

#[derive(SpacetimeType, Debug, Eq, PartialEq)]
pub enum UserRight {
    UnitSync,
}

const SERVER_IDENTITY_HEX: &str =
    "41e46206c21c35062daafb836aacbebca4444370e9eef8ec8452003d8d6f0ad1";
pub const LOCAL_IDENTITY_HEX: &str =
    "5b32d445d4258388a7f8e337acc0d6f9a56c4799298aa6ee49906876cf98c066";

#[spacetimedb(reducer)]
fn give_right(ctx: ReducerContext, identity: String) -> Result<(), String> {
    if !hex::encode(ctx.sender.as_bytes()).eq(SERVER_IDENTITY_HEX) {
        return Err("Sender identity doesn't match server".to_owned());
    }
    let identity = Identity::from_str(&identity).map_err(|e| e.to_string())?;
    let right = UserRight::UnitSync;
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

impl UserAccess {
    pub fn init() -> Result<(), String> {
        UserAccess::insert(UserAccess {
            identity: Identity::from_str(LOCAL_IDENTITY_HEX).unwrap(),
            rights: [UserRight::UnitSync].into(),
        })?;
        Ok(())
    }
}
