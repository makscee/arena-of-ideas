use spacetimedb::{spacetimedb, Identity, ReducerContext};

#[spacetimedb(table)]
pub struct User {
    #[primarykey]
    identity: Identity,
    name: Option<String>,
    email: Option<String>,
}

#[spacetimedb(reducer)]
pub fn add_user(ctx: ReducerContext) -> Result<(), String> {
    if User::filter_by_identity(&ctx.sender).is_some() {
        Err("User already added".to_string())
    } else {
        User::insert(User {
            identity: ctx.sender,
            name: None,
            email: None,
        })?;
        Ok(())
    }
}

#[spacetimedb(reducer)]
/// Clientss invoke this reducer to set their user names.
pub fn set_name(ctx: ReducerContext, name: String) -> Result<(), String> {
    let name = validate_name(name)?;
    if let Some(user) = User::filter_by_identity(&ctx.sender) {
        User::update_by_identity(
            &ctx.sender,
            User {
                name: Some(name),
                ..user
            },
        );
        Ok(())
    } else {
        Err("Cannot set name for unknown user".to_string())
    }
}

/// Takes a name and checks if it's acceptable as a user's name.
fn validate_name(name: String) -> Result<String, String> {
    if name.is_empty() {
        Err("Names must not be empty".to_string())
    } else {
        Ok(name)
    }
}
