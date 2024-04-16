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
    pass_hash: Option<String>,
    online: bool,
    last_login: Timestamp,
}

#[spacetimedb(reducer)]
fn register_empty(ctx: ReducerContext) -> Result<(), String> {
    User::clear_identity(&ctx.sender);
    let user = User {
        id: 0,
        identities: vec![ctx.sender],
        name: format!("player#{}", User::iter().count()),
        pass_hash: None,
        online: false,
        last_login: Timestamp::UNIX_EPOCH,
    };
    User::insert(user)?;
    Ok(())
}

#[spacetimedb(reducer)]
fn register(ctx: ReducerContext, name: String, pass: String) -> Result<(), String> {
    let name = User::validate_name(name)?;
    let pass_hash = Some(User::hash_pass(pass)?);
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
    let mut user = User::filter_by_name(&name).context_str("User not found")?;
    if user.pass_hash.is_none() {
        return Err("No password set for user".to_owned());
    }
    if !user.check_pass(pass) {
        Err("Wrong password".to_owned())
    } else {
        if let Ok(mut user) = User::find_by_identity(&ctx.sender) {
            user.online = false;
            user.remove_identity(&ctx.sender);
            User::update_by_id(&user.id.clone(), user);
        }
        if !user.identities.contains(&ctx.sender) {
            User::clear_identity(&ctx.sender);
            user.identities.push(ctx.sender);
        }
        user.login();
        Ok(())
    }
}

#[spacetimedb(reducer)]
fn login_by_identity(ctx: ReducerContext) -> Result<(), String> {
    let user = User::find_by_identity(&ctx.sender)?;
    user.login();
    Ok(())
}

#[spacetimedb(reducer)]
fn logout(ctx: ReducerContext) -> Result<(), String> {
    let mut user = User::find_by_identity(&ctx.sender)?;
    user.online = false;
    user.remove_identity(&ctx.sender);
    User::update_by_id(&user.id.clone(), user);
    Ok(())
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

#[spacetimedb(reducer)]
fn set_password(ctx: ReducerContext, old_pass: String, new_pass: String) -> Result<(), String> {
    if let Ok(user) = User::find_by_identity(&ctx.sender) {
        if !user.check_pass(old_pass) {
            return Err("Old password did not match".to_owned());
        }
        let pass_hash = Some(User::hash_pass(new_pass)?);
        User::update_by_id(&user.id, User { pass_hash, ..user });
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

    fn check_pass(&self, pass: String) -> bool {
        if let Some(hash) = &self.pass_hash {
            bcrypt::verify(pass, hash)
        } else {
            true
        }
    }

    fn hash_pass(pass: String) -> Result<String, String> {
        bcrypt::hash(pass).map_err(|e| e.to_string())
    }

    pub fn find_by_identity(identity: &Identity) -> Result<User, String> {
        User::iter()
            .find(|u| u.identities.contains(identity))
            .context_str("User not found")
    }

    fn login(mut self) {
        self.online = true;
        self.last_login = Timestamp::now();
        User::update_by_id(&self.id.clone(), self);
    }

    fn clear_identity(identity: &Identity) {
        if let Ok(mut user) = User::find_by_identity(identity) {
            user.remove_identity(identity);
            User::update_by_id(&user.id.clone(), user);
        }
    }

    fn remove_identity(&mut self, identity: &Identity) {
        self.identities.retain(|i| !i.eq(identity));
    }
}
