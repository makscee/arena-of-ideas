use bcrypt_no_getrandom::{hash_with_salt, verify};
use rand::RngCore;
use schema::OptionExpressionError;
use spacetimedb::Timestamp;

use super::*;

#[reducer]
fn register(ctx: &ReducerContext, name: String, pass: String) -> Result<(), String> {
    let name = Player::validate_name(ctx, name)?;
    let pass_hash = Some(Player::hash_pass(ctx, pass)?);
    Player::clear_identity(ctx, &ctx.sender);
    let mut player = Player::new(name);
    player.id = Some(next_id(ctx));
    player.player_data = Some(PlayerData::new(pass_hash, false, 0));
    player.identity = Some(PlayerIdentity::new(Some(ctx.identity().to_string())));
    player.set_parent(ctx, 0);
    Ok(())
}

#[reducer]
fn login(ctx: &ReducerContext, name: String, pass: String) -> Result<(), String> {
    let mut player = Player::get_by_data(ctx, &format!("\"{name}\""))
        .to_e_s("Player not found")?
        .with_components(ctx);
    if player.player_data().pass_hash.is_none() {
        return Err("No password set for player".to_owned());
    }
    if !player.check_pass(pass) {
        Err("Wrong name or password".to_owned())
    } else {
        if let Ok(player) = ctx.player() {
            let mut player = player.logout();
            player.identity_mut().data = None;
            player.save(ctx);
        }
        player.identity_mut().data = Some(ctx.identity().to_string());
        player.login().save(ctx);
        Ok(())
    }
}

#[reducer]
fn login_by_identity(ctx: &ReducerContext) -> Result<(), String> {
    ctx.player()?.login().save(ctx);
    Ok(())
}

#[reducer]
fn logout(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?.logout();
    player.identity_mut().data = None;
    player.save(ctx);
    Ok(())
}

#[reducer]
fn set_password(ctx: &ReducerContext, old_pass: String, new_pass: String) -> Result<(), String> {
    let mut player = ctx.player()?;
    if !player.check_pass(old_pass) {
        return Err("Old password did not match".to_owned());
    }
    player.player_data_mut().pass_hash = Some(Player::hash_pass(ctx, new_pass)?);
    player.save(ctx);
    Ok(())
}

#[reducer(client_disconnected)]
fn identity_disconnected(ctx: &ReducerContext) {
    if let Ok(player) = ctx.player() {
        player.logout().save(ctx);
    }
}

impl Player {
    fn validate_name(ctx: &ReducerContext, name: String) -> Result<String, String> {
        let name = name.to_lowercase();
        if name.is_empty() {
            Err("Names must not be empty".to_string())
        } else if let Some(c) = name.chars().find(|c| !c.is_alphanumeric()) {
            Err(format!("Wrong character: {c}"))
        } else if ctx.db.tnodes().data().find(&name).is_some() {
            Err(format!("Name is taken"))
        } else {
            Ok(name)
        }
    }
    fn check_pass(&self, pass: String) -> bool {
        if let Some(hash) = &self.player_data().pass_hash {
            match verify(pass, hash) {
                Ok(v) => v,
                Err(e) => {
                    log::error!("Password verify error: {e}");
                    false
                }
            }
        } else {
            true
        }
    }
    fn hash_pass(ctx: &ReducerContext, pass: String) -> Result<String, String> {
        let mut salt = [0u8; 16];
        ctx.rng().fill_bytes(&mut salt);
        match hash_with_salt(pass, 13, salt) {
            Ok(hash) => Ok(hash.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }
    pub fn find_by_identity(ctx: &ReducerContext, identity: &Identity) -> Option<Player> {
        let i = PlayerIdentity::get_by_data(ctx, &identity.to_hex().to_string())?;
        Player::find_parent(ctx, i.id())
    }
    fn login(mut self) -> Self {
        let data = self.player_data_mut();
        data.last_login = Timestamp::now().into_micros_since_epoch();
        data.online = true;
        self
    }
    fn logout(mut self) -> Self {
        self.player_data_mut().online = false;
        self
    }
    fn clear_identity(ctx: &ReducerContext, identity: &Identity) {
        if let Some(mut player) = Self::find_by_identity(ctx, identity) {
            player.identity = None;
            player.save(ctx);
        }
    }
}

pub trait GetPlayer {
    fn player(&self) -> Result<Player, String>;
}

impl GetPlayer for ReducerContext {
    fn player(&self) -> Result<Player, String> {
        Player::find_by_identity(self, &self.sender).to_e_s("Player not found")
    }
}
