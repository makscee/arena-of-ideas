use bcrypt_no_getrandom::{hash_with_salt, verify};
use rand::RngCore;
use spacetimedb::Timestamp;

use super::*;

#[spacetimedb(table(public))]
pub struct TPlayer {
    #[primarykey]
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
    TPlayer::clear_identity(&ctx.sender);
    let id = next_id();
    let player = TPlayer {
        id,
        identities: vec![ctx.sender],
        name: format!("player#{}", id),
        pass_hash: None,
        online: false,
        last_login: Timestamp::UNIX_EPOCH,
    };
    TPlayer::insert(player)?;
    TWallet::new(id)?;
    Ok(())
}

#[spacetimedb(reducer)]
fn register(ctx: ReducerContext, name: String, pass: String) -> Result<(), String> {
    let name = TPlayer::validate_name(name)?;
    let pass_hash = Some(TPlayer::hash_pass(pass)?);
    TPlayer::clear_identity(&ctx.sender);
    let id = next_id();
    TPlayer::insert(TPlayer {
        id,
        identities: vec![ctx.sender],
        name,
        pass_hash,
        online: false,
        last_login: Timestamp::UNIX_EPOCH,
    })?;
    GlobalEvent::Register.post(id);
    Ok(())
}

#[spacetimedb(reducer)]
fn login(ctx: ReducerContext, name: String, pass: String) -> Result<(), String> {
    let mut player = TPlayer::filter_by_name(&name).context_str("Wrong name or password")?;
    if player.pass_hash.is_none() {
        return Err("No password set for player".to_owned());
    }
    if !player.check_pass(pass) {
        Err("Wrong name or password".to_owned())
    } else {
        if let Ok(mut player) = ctx.player() {
            player.logout();
            player.remove_identity(&ctx.sender);
            TPlayer::update_by_id(&player.id.clone(), player);
        }
        if !player.identities.contains(&ctx.sender) {
            TPlayer::clear_identity(&ctx.sender);
            player.identities.push(ctx.sender);
        }
        player.login();
        Ok(())
    }
}

#[spacetimedb(reducer)]
fn login_by_identity(ctx: ReducerContext) -> Result<(), String> {
    let player = ctx.player()?;
    player.login();
    Ok(())
}

#[spacetimedb(reducer)]
fn logout(ctx: ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    player.logout();
    GlobalEvent::LogOut.post(player.id);
    player.remove_identity(&ctx.sender);
    TPlayer::update_by_id(&player.id.clone(), player);
    Ok(())
}

#[spacetimedb(reducer)]
fn set_name(ctx: ReducerContext, name: String) -> Result<(), String> {
    let name = TPlayer::validate_name(name)?;
    if let Ok(player) = ctx.player() {
        TPlayer::update_by_id(&player.id, TPlayer { name, ..player });
        Ok(())
    } else {
        Err("Cannot set name for unknown player".to_string())
    }
}

#[spacetimedb(reducer)]
fn set_password(ctx: ReducerContext, old_pass: String, new_pass: String) -> Result<(), String> {
    if let Ok(player) = ctx.player() {
        if !player.check_pass(old_pass) {
            return Err("Old password did not match".to_owned());
        }
        let pass_hash = Some(TPlayer::hash_pass(new_pass)?);
        TPlayer::update_by_id(
            &player.id,
            TPlayer {
                pass_hash,
                ..player
            },
        );
        Ok(())
    } else {
        Err("Cannot set name for unknown player".to_string())
    }
}

#[spacetimedb(disconnect)]
fn identity_disconnected(ctx: ReducerContext) {
    if let Ok(mut player) = ctx.player() {
        GlobalEvent::LogOut.post(player.id);
        player.logout();
        TPlayer::update_by_id(&player.id.clone(), player);
    }
}

impl TPlayer {
    fn validate_name(name: String) -> Result<String, String> {
        if name.is_empty() {
            Err("Names must not be empty".to_string())
        } else if TPlayer::filter_by_name(&name).is_some() {
            Err("Name is taken".to_string())
        } else {
            Ok(name)
        }
    }
    fn check_pass(&self, pass: String) -> bool {
        if let Some(hash) = &self.pass_hash {
            match verify(pass, hash) {
                Ok(v) => v,
                Err(e) => {
                    self::eprintln!("Password verify error: {e}");
                    false
                }
            }
        } else {
            true
        }
    }
    fn hash_pass(pass: String) -> Result<String, String> {
        let mut salt = [0u8; 16];
        rng().fill_bytes(&mut salt);
        match hash_with_salt(pass, 13, salt) {
            Ok(hash) => Ok(hash.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }
    pub fn find_by_identity(identity: &Identity) -> Result<TPlayer, String> {
        TPlayer::iter()
            .find(|u| u.identities.contains(identity))
            .context_str("Player not found")
    }
    fn login(mut self) {
        self.online = true;
        self.last_login = Timestamp::now();
        GlobalEvent::LogIn.post(self.id);
        TPlayer::update_by_id(&self.id.clone(), self);
    }
    fn logout(&mut self) {
        self.online = false;
        if self.last_login.into_micros_since_epoch() > 0 {
            TPlayerStats::register_time_played(
                self.id,
                Timestamp::now()
                    .duration_since(self.last_login)
                    .unwrap()
                    .as_micros() as u64,
            );
        }
    }
    fn clear_identity(identity: &Identity) {
        if let Ok(mut player) = TPlayer::find_by_identity(identity) {
            player.remove_identity(identity);
            TPlayer::update_by_id(&player.id.clone(), player);
        }
    }
    fn remove_identity(&mut self, identity: &Identity) {
        self.identities.retain(|i| !i.eq(identity));
    }
    pub fn cleanup() {
        for player in TPlayer::iter() {
            if player.identities.is_empty() && player.pass_hash.is_none() {
                player.delete();
            }
        }
    }
}

pub trait GetPlayer {
    fn player(&self) -> Result<TPlayer, String>;
}

impl GetPlayer for ReducerContext {
    fn player(&self) -> Result<TPlayer, String> {
        TPlayer::find_by_identity(&self.sender)
    }
}
