use pwhash::bcrypt;
use spacetimedb::Timestamp;

use super::*;

#[spacetimedb(table)]
pub struct User {
    #[primarykey]
    #[autoinc]
    pub id: u64,
    #[unique]
    pub name: String,
    identities: Vec<Identity>,
    pass_hash: String,
    online: bool,
    last_login: Timestamp,
}

#[spacetimedb(reducer)]
fn register(ctx: ReducerContext, name: String, pass: String) -> Result<(), String> {
    let name = User::validate_name(name)?;
    let pass_hash = bcrypt::hash(pass).map_err(|e| e.to_string())?;
    User::clear_identity(&ctx.sender);
    User::insert(User {
        id: 0,
        identities: vec![ctx.sender],
        name,
        pass_hash,
        online: false,
        last_login: Timestamp::UNIX_EPOCH,
    })?;
    Ok(())
}

#[spacetimedb(reducer)]
fn login(ctx: ReducerContext, name: String, pass: String) -> Result<(), String> {
    let mut user = User::filter_by_name(&name)
        .context("User not found")
        .map_err(|e| e.to_string())?;
    if !bcrypt::verify(pass, &user.pass_hash) {
        Err("Wrong password".to_owned())
    } else {
        if !user.identities.contains(&ctx.sender) {
            User::clear_identity(&ctx.sender);
            user.identities.push(ctx.sender);
        }
        user.login();
        Ok(())
    }
}

#[spacetimedb(reducer)]
pub fn login_by_identity(ctx: ReducerContext, name: String) -> Result<(), String> {
    let user = User::filter_by_name(&name)
        .context("User not found")
        .map_err(|e| e.to_string())?;
    if !user.identities.contains(&ctx.sender) {
        Err("Identity not connected to user name".to_string())
    } else {
        user.login();
        Ok(())
    }
}

#[spacetimedb(reducer)]
fn set_name(ctx: ReducerContext, name: String) -> Result<(), String> {
    let name = User::validate_name(name)?;
    if let Ok(user) = User::find_by_identity(&ctx.sender) {
        User::update_by_id(&user.id, User { name, ..user });
        Ok(())
    } else {
        Err("Cannot set name for unknown user".to_string())
    }
}

#[spacetimedb(disconnect)]
fn identity_disconnected(ctx: ReducerContext) {
    if let Ok(mut user) = User::find_by_identity(&ctx.sender) {
        user.online = false;
        User::update_by_id(&user.id.clone(), user);
    }
}

impl User {
    fn validate_name(name: String) -> Result<String, String> {
        if name.is_empty() {
            Err("Names must not be empty".to_string())
        } else if User::filter_by_name(&name).is_some() {
            Err("Name is taken".to_string())
        } else {
            Ok(name)
        }
    }

    pub fn find_by_identity(identity: &Identity) -> Result<User, String> {
        User::iter()
            .find(|u| u.identities.contains(identity))
            .context("User not found")
            .map_err(|e| e.to_string())
    }

    fn login(mut self) {
        self.online = true;
        self.last_login = Timestamp::now();
        User::update_by_id(&self.id.clone(), self);
    }

    fn clear_identity(identity: &Identity) {
        if let Ok(mut user) = User::find_by_identity(identity) {
            user.identities.retain(|i| !i.eq(identity));
            User::update_by_id(&user.id.clone(), user);
        }
    }
}
