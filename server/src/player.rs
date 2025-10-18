use bcrypt_no_getrandom::{hash_with_salt, verify};
use spacetimedb::rand::RngCore;

use super::*;

use schema::NodeKind;

#[reducer]
fn register(ctx: &ReducerContext, name: String, pass: String) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let name = NPlayer::validate_name(ctx, name)?;
    let pass_hash = Some(NPlayer::hash_pass(ctx, pass)?);
    NPlayer::clear_identity(ctx, &ctx.rctx().sender);
    let mut player = NPlayer::new(ctx.next_id(), name);
    player
        .player_data
        .state_mut()
        .set(NPlayerData::new(ctx.next_id(), pass_hash, false, 0).with_owner(player.id));
    player.identity.state_mut().set(
        NPlayerIdentity::new(ctx.next_id(), Some(ctx.rctx().sender.to_string()))
            .with_owner(player.id),
    );
    player.save(ctx)?;
    Ok(())
}

#[reducer]
fn login(ctx: &ReducerContext, name: String, pass: String) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = NPlayer::find_by_data(ctx, &NPlayer::new(0, name).get_data())
        .to_custom_e_s("Player not found")?;
    debug!("{player:?}");
    if player.player_data_load(ctx).track()?.pass_hash.is_none() {
        return Err("No password set for player".to_owned());
    }
    if !player.check_pass(pass) {
        Err("Wrong name or password".to_owned())
    } else {
        NPlayer::clear_identity(ctx, &ctx.rctx().sender);
        let identity =
            NPlayerIdentity::new(player.id, Some(ctx.rctx().sender.to_string())).insert(ctx);
        player.identity.state_mut().set(identity);
        player.login(ctx)?.save(ctx)?;
        Ok(())
    }
}

#[reducer]
fn login_by_identity(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    ctx.player()?.login(ctx)?.take().save(ctx)?;
    Ok(())
}

#[reducer]
fn logout(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?.logout(ctx)?;
    player.identity_load(ctx)?.delete(ctx);
    player.take().save(ctx)?;
    Ok(())
}

#[reducer]
fn set_password(ctx: &ReducerContext, old_pass: String, new_pass: String) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    if !player.check_pass(old_pass) {
        return Err("Old password did not match".to_owned());
    }
    if let Ok(player_data) = player.player_data.get_mut() {
        player_data.pass_hash = Some(NPlayer::hash_pass(ctx, new_pass)?);
    }
    player.take().save(ctx)?;
    Ok(())
}

#[reducer(client_disconnected)]
fn identity_disconnected(ctx: &ReducerContext) {
    let ctx = &mut ctx.as_context();
    if let Ok(player) = ctx.player() {
        player.logout(ctx).unwrap().save(ctx).log();
    }
}

impl NPlayer {
    fn validate_name(ctx: &ServerContext, name: String) -> Result<String, String> {
        let name = name.to_lowercase();
        if name.is_empty() {
            Err("Names must not be empty".to_string())
        } else if let Some(c) = name.chars().find(|c| !c.is_alphanumeric()) {
            Err(format!("Wrong character: {c}"))
        } else if NPlayer::find_by_data(ctx, &NPlayer::new(0, name.clone()).get_data()).is_some() {
            Err(format!("Name is taken"))
        } else {
            Ok(name)
        }
    }
    fn check_pass(&self, pass: String) -> bool {
        if let Some(player_data) = self.player_data.get() {
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
    fn hash_pass(ctx: &ServerContext, pass: String) -> Result<String, String> {
        let mut salt = [0u8; 16];
        ctx.rng().fill_bytes(&mut salt);
        match hash_with_salt(pass, 13, salt) {
            Ok(hash) => Ok(hash.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }
    pub fn find_identity(ctx: &ServerContext, identity: &Identity) -> Option<NPlayerIdentity> {
        NPlayerIdentity::find_by_data(ctx, &ron::to_string(&Some(identity.to_string())).unwrap())
            .into_iter()
            .next()
    }
    fn login(mut self, ctx: &ServerContext) -> Result<Self, String> {
        let ts = ctx.rctx().timestamp.to_micros_since_unix_epoch() as u64;
        let data = self.player_data_load(ctx)?;
        debug!("{data:?}");
        data.last_login = ts;
        data.online = true;
        Ok(self)
    }
    fn logout(mut self, ctx: &ServerContext) -> Result<Self, String> {
        self.player_data_load(ctx)?.online = false;
        Ok(self)
    }
    fn clear_identity(ctx: &ServerContext, identity: &Identity) {
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
        let identity = NPlayer::find_identity(self, &self.source().rctx().sender)
            .to_custom_e_s("NPlayerIdentity not found")?;
        let id = self
            .get_parents_of_kind(identity.id, NodeKind::NPlayer)?
            .into_iter()
            .next()
            .to_not_found()?;
        NPlayer::load(self.source(), id)
    }
}
