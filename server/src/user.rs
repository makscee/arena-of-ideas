use pwhash::bcrypt;
use spacetimedb::Timestamp;

use super::*;

#[spacetimedb(table)]
pub struct TUser {
    #[primarykey]
    pub id: GID,
    #[unique]
    pub name: String,
    identities: Vec<Identity>,
    pass_hash: Option<String>,
    online: bool,
    last_login: Timestamp,
}

#[spacetimedb(reducer)]
fn register_empty(ctx: ReducerContext) -> Result<(), String> {
    TUser::clear_identity(&ctx.sender);
    let id = next_id();
    let user = TUser {
        id,
        identities: vec![ctx.sender],
        name: format!("player#{}", id),
        pass_hash: None,
        online: false,
        last_login: Timestamp::UNIX_EPOCH,
    };
    TUser::insert(user)?;
    TWallet::new(id)?;
    Ok(())
}

#[spacetimedb(reducer)]
fn register(ctx: ReducerContext, name: String, pass: String) -> Result<(), String> {
    let name = TUser::validate_name(name)?;
    let pass_hash = Some(TUser::hash_pass(pass)?);
    TUser::clear_identity(&ctx.sender);
    TUser::insert(TUser {
        id: next_id(),
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
    let mut user = TUser::filter_by_name(&name).context_str("Wrong name or password")?;
    if user.pass_hash.is_none() {
        return Err("No password set for user".to_owned());
    }
    if !user.check_pass(pass) {
        Err("Wrong name or password".to_owned())
    } else {
        if let Ok(mut user) = TUser::find_by_identity(&ctx.sender) {
            user.online = false;
            user.remove_identity(&ctx.sender);
            TUser::update_by_id(&user.id.clone(), user);
        }
        if !user.identities.contains(&ctx.sender) {
            TUser::clear_identity(&ctx.sender);
            user.identities.push(ctx.sender);
        }
        user.login();
        Ok(())
    }
}

#[spacetimedb(reducer)]
fn login_by_identity(ctx: ReducerContext) -> Result<(), String> {
    let user = TUser::find_by_identity(&ctx.sender)?;
    user.login();
    Ok(())
}

#[spacetimedb(reducer)]
fn logout(ctx: ReducerContext) -> Result<(), String> {
    let mut user = TUser::find_by_identity(&ctx.sender)?;
    user.online = false;
    user.remove_identity(&ctx.sender);
    TUser::update_by_id(&user.id.clone(), user);
    Ok(())
}

#[spacetimedb(reducer)]
fn set_name(ctx: ReducerContext, name: String) -> Result<(), String> {
    let name = TUser::validate_name(name)?;
    if let Ok(user) = TUser::find_by_identity(&ctx.sender) {
        TUser::update_by_id(&user.id, TUser { name, ..user });
        Ok(())
    } else {
        Err("Cannot set name for unknown user".to_string())
    }
}

#[spacetimedb(reducer)]
fn set_password(ctx: ReducerContext, old_pass: String, new_pass: String) -> Result<(), String> {
    if let Ok(user) = TUser::find_by_identity(&ctx.sender) {
        if !user.check_pass(old_pass) {
            return Err("Old password did not match".to_owned());
        }
        let pass_hash = Some(TUser::hash_pass(new_pass)?);
        TUser::update_by_id(&user.id, TUser { pass_hash, ..user });
        Ok(())
    } else {
        Err("Cannot set name for unknown user".to_string())
    }
}

#[spacetimedb(disconnect)]
fn identity_disconnected(ctx: ReducerContext) {
    if let Ok(mut user) = TUser::find_by_identity(&ctx.sender) {
        user.online = false;
        TUser::update_by_id(&user.id.clone(), user);
    }
}

impl TUser {
    fn validate_name(name: String) -> Result<String, String> {
        if name.is_empty() {
            Err("Names must not be empty".to_string())
        } else if TUser::filter_by_name(&name).is_some() {
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

    pub fn find_by_identity(identity: &Identity) -> Result<TUser, String> {
        TUser::iter()
            .find(|u| u.identities.contains(identity))
            .context_str("User not found")
    }

    fn login(mut self) {
        self.online = true;
        self.last_login = Timestamp::now();
        TUser::update_by_id(&self.id.clone(), self);
    }

    fn clear_identity(identity: &Identity) {
        if let Ok(mut user) = TUser::find_by_identity(identity) {
            user.remove_identity(identity);
            TUser::update_by_id(&user.id.clone(), user);
        }
    }

    fn remove_identity(&mut self, identity: &Identity) {
        self.identities.retain(|i| !i.eq(identity));
    }
}
