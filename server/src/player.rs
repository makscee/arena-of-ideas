use bcrypt_no_getrandom::{hash_with_salt, verify};
use rand::RngCore;

use super::*;

use schema::NodeKind;

#[reducer]
fn register(ctx: &ReducerContext, name: String, pass: String) -> Result<(), String> {
    let name = NPlayer::validate_name(ctx, name)?;
    let pass_hash = Some(NPlayer::hash_pass(ctx, pass)?);
    NPlayer::clear_identity(ctx, &ctx.sender);
    let mut player = NPlayer::new(0, name).insert(ctx);
    let player_data = NPlayerData::new(player.id, pass_hash, false, 0).insert(ctx);
    player.player_data_set(ctx, player_data)?;
    let identity = NPlayerIdentity::new(player.id, Some(ctx.sender.to_string())).insert(ctx);
    player.identity_set(ctx, identity)?;
    Ok(())
}

#[reducer]
fn login(ctx: &ReducerContext, name: String, pass: String) -> Result<(), String> {
    let mut player = NPlayer::find_by_data(ctx, name.clone()).to_custom_e_s("Player not found")?;
    debug!("{player:?}");
    if player.player_data_load(ctx)?.pass_hash.is_none() {
        return Err("No password set for player".to_owned());
    }
    if !player.check_pass(pass) {
        Err("Wrong name or password".to_owned())
    } else {
        NPlayer::clear_identity(ctx, &ctx.sender);
        let identity = NPlayerIdentity::new(player.id, Some(ctx.sender.to_string())).insert(ctx);
        player.identity_set(ctx, identity)?;
        player.login(ctx)?.save(ctx);
        Ok(())
    }
}

#[reducer]
fn login_by_identity(ctx: &ReducerContext) -> Result<(), String> {
    ctx.player()?.login(ctx)?.save(ctx);
    Ok(())
}

#[reducer]
fn logout(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?.logout(ctx)?;
    player.identity_load(ctx)?.delete(ctx);
    player.save(ctx);
    Ok(())
}

#[reducer]
fn set_password(ctx: &ReducerContext, old_pass: String, new_pass: String) -> Result<(), String> {
    let mut player = ctx.player()?;
    if !player.check_pass(old_pass) {
        return Err("Old password did not match".to_owned());
    }
    if let Some(player_data) = player.player_data.get_data_mut() {
        player_data.pass_hash = Some(NPlayer::hash_pass(ctx, new_pass)?);
    }
    player.save(ctx);
    Ok(())
}

#[reducer(client_disconnected)]
fn identity_disconnected(ctx: &ReducerContext) {
    if let Ok(player) = ctx.player() {
        player.logout(ctx).unwrap().save(ctx);
    }
}

impl NPlayer {
    fn validate_name(ctx: &ReducerContext, name: String) -> Result<String, String> {
        let name = name.to_lowercase();
        if name.is_empty() {
            Err("Names must not be empty".to_string())
        } else if let Some(c) = name.chars().find(|c| !c.is_alphanumeric()) {
            Err(format!("Wrong character: {c}"))
        } else if NPlayer::find_by_data(ctx, name.clone()).is_some() {
            Err(format!("Name is taken"))
        } else {
            Ok(name)
        }
    }
    fn check_pass(&self, pass: String) -> bool {
        if let Some(player_data) = self.player_data.get_data() {
            if let Some(pass_hash) = &player_data.pass_hash {
                match verify(pass, pass_hash) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("Password verify error: {e}");
                        false
                    }
                }
            } else {
                true
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
    pub fn find_identity(ctx: &ServerContext, identity: &Identity) -> Option<NPlayerIdentity> {
        NPlayerIdentity::find_by_data(ctx, Some(identity.to_string()))
    }
    fn login(mut self, ctx: &ReducerContext) -> Result<Self, String> {
        let ctx = ctx.as_context();
        let data = self.player_data_load(ctx)?;
        debug!("{data:?}");
        data.last_login = ctx.timestamp.to_micros_since_unix_epoch() as u64;
        data.online = true;
        Ok(self)
    }
    fn logout(mut self, ctx: &ReducerContext) -> Result<Self, String> {
        let ctx = ctx.as_context();
        self.player_data_load(ctx)?.online = false;
        Ok(self)
    }
    fn clear_identity(ctx: &ReducerContext, identity: &Identity) {
        if let Some(node) = Self::find_identity(ctx, identity) {
            info!("identity cleared for {node:?}");
            node.delete(ctx);
        }
    }
}

pub trait GetPlayer {
    fn player(&self) -> NodeResult<NPlayer>;
}

impl GetPlayer for ServerContext<'_> {
    fn player(&self) -> NodeResult<NPlayer> {
        let identity = NPlayer::find_identity(self, &self.source().reducer_context().sender)
            .to_custom_e_s("NPlayerIdentity not found")?;
        let id = identity
            .id
            .find_kind_child(self, NodeKind::NPlayer)
            .to_custom_e_s("Failed to find Player by Identity")?;
        NPlayer::load(self, id)
    }
}
