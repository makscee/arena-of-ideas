use super::*;

#[spacetimedb(table)]
pub struct User {
    #[primarykey]
    identity: Identity,
    name: Option<String>,
    email: Option<String>,
}

#[spacetimedb(reducer)]
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

#[spacetimedb(reducer)]
pub fn set_email(ctx: ReducerContext, email: String) -> Result<(), String> {
    let email = validate_email(email)?;
    if let Some(user) = User::filter_by_identity(&ctx.sender) {
        User::update_by_identity(
            &ctx.sender,
            User {
                email: Some(email),
                ..user
            },
        );
        Ok(())
    } else {
        Err("Cannot set email for unknown user".to_string())
    }
}

fn validate_name(name: String) -> Result<String, String> {
    if name.is_empty() {
        Err("Names must not be empty".to_string())
    } else {
        Ok(name)
    }
}

fn validate_email(email: String) -> Result<String, String> {
    if email.contains("@") {
        Ok(email)
    } else {
        Err("Wrong email format".to_string())
    }
}

#[spacetimedb(connect)]
pub fn identity_connected(ctx: ReducerContext) {
    log::debug!("Identity connected {:?}", ctx.sender);
    if let Some(_) = User::filter_by_identity(&ctx.sender) {
        // User::update_by_identity(&ctx.sender, User { online: true, ..user });
    } else {
        User::insert(User {
            identity: ctx.sender,
            name: None,
            email: None,
        })
        .unwrap();
    }
}
