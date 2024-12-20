use bcrypt_no_getrandom::{hash_with_salt, verify};
use rand::RngCore;
use spacetimedb::Timestamp;

use super::*;

#[spacetimedb::table(public, name = player)]
pub struct TPlayer {
    #[primary_key]
    pub id: u64,
    #[unique]
    pub name: String,
    identities: Vec<Identity>,
    pass_hash: Option<String>,
    online: bool,
    last_login: Timestamp,
}

#[spacetimedb::reducer]
fn register_empty(ctx: &ReducerContext) -> Result<(), String> {
    TPlayer::clear_identity(ctx, &ctx.sender);
    let id = next_id(ctx);
    let player = TPlayer {
        id,
        identities: vec![ctx.sender],
        name: format!("player#{}", id),
        pass_hash: None,
        online: false,
        last_login: Timestamp::UNIX_EPOCH,
    };
    ctx.db.player().insert(player);
    TWallet::new(ctx, id)?;
    Ok(())
}

#[spacetimedb::reducer]
fn register(ctx: &ReducerContext, name: String, pass: String) -> Result<(), String> {
    let name = TPlayer::validate_name(ctx, name)?;
    let pass_hash = Some(TPlayer::hash_pass(ctx, pass)?);
    TPlayer::clear_identity(ctx, &ctx.sender);
    let id = next_id(ctx);
    ctx.db.player().insert(TPlayer {
        id,
        identities: vec![ctx.sender],
        name,
        pass_hash,
        online: false,
        last_login: Timestamp::UNIX_EPOCH,
    });
    Ok(())
}

#[spacetimedb::reducer]
fn login(ctx: &ReducerContext, name: String, pass: String) -> Result<(), String> {
    let mut player = ctx
        .db
        .player()
        .name()
        .find(&name)
        .context_str("Wrong name or password")?;
    if player.pass_hash.is_none() {
        return Err("No password set for player".to_owned());
    }
    if !player.check_pass(pass) {
        Err("Wrong name or password".to_owned())
    } else {
        if let Ok(mut player) = ctx.player() {
            player.logout(ctx);
            player.remove_identity(&ctx.sender);
            ctx.db.player().id().update(player);
        }
        if !player.identities.contains(&ctx.sender) {
            TPlayer::clear_identity(ctx, &ctx.sender);
            player.identities.push(ctx.sender);
        }
        player.login(ctx);
        Ok(())
    }
}

#[spacetimedb::reducer]
fn login_by_identity(ctx: &ReducerContext) -> Result<(), String> {
    let player = ctx.player()?;
    player.login(ctx);
    Ok(())
}

#[spacetimedb::reducer]
fn logout(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    player.logout(ctx);
    player.remove_identity(&ctx.sender);
    ctx.db.player().id().update(player);
    Ok(())
}

#[spacetimedb::reducer]
fn set_name(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let name = TPlayer::validate_name(ctx, name)?;
    if let Ok(player) = ctx.player() {
        ctx.db.player().id().update(TPlayer { name, ..player });
        Ok(())
    } else {
        Err("Cannot set name for unknown player".to_string())
    }
}

#[spacetimedb::reducer]
fn set_password(ctx: &ReducerContext, old_pass: String, new_pass: String) -> Result<(), String> {
    if let Ok(player) = ctx.player() {
        if !player.check_pass(old_pass) {
            return Err("Old password did not match".to_owned());
        }
        let pass_hash = Some(TPlayer::hash_pass(ctx, new_pass)?);
        ctx.db.player().id().update(TPlayer {
            pass_hash,
            ..player
        });
        Ok(())
    } else {
        Err("Cannot set name for unknown player".to_string())
    }
}

#[spacetimedb::reducer(client_disconnected)]
fn identity_disconnected(ctx: &ReducerContext) {
    if let Ok(mut player) = ctx.player() {
        player.logout(ctx);
        ctx.db.player().id().update(player);
    }
}

impl TPlayer {
    pub fn check_owner(&self, id: u64) -> Result<(), String> {
        if self.id != id {
            Err(format!("Player is not owner of {id}"))
        } else {
            Ok(())
        }
    }
    fn validate_name(ctx: &ReducerContext, name: String) -> Result<String, String> {
        if name.is_empty() {
            Err("Names must not be empty".to_string())
        } else if ctx.db.player().name().find(&name).is_some() {
            Err("Name is taken".to_string())
        } else if let Some(c) = name.chars().find(|c| !c.is_alphanumeric()) {
            Err(format!("Wrong character: {c}"))
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
    fn hash_pass(ctx: &ReducerContext, pass: String) -> Result<String, String> {
        let mut salt = [0u8; 16];
        ctx.rng().fill_bytes(&mut salt);
        match hash_with_salt(pass, 13, salt) {
            Ok(hash) => Ok(hash.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }
    pub fn find_by_identity(ctx: &ReducerContext, identity: &Identity) -> Result<TPlayer, String> {
        ctx.db
            .player()
            .iter()
            .find(|u| u.identities.contains(identity))
            .context_str("Player not found")
    }
    fn login(mut self, ctx: &ReducerContext) {
        self.online = true;
        self.last_login = Timestamp::now();
        ctx.db.player().id().update(self);
    }
    fn logout(&mut self, _: &ReducerContext) {
        self.online = false;
    }
    fn clear_identity(ctx: &ReducerContext, identity: &Identity) {
        if let Ok(mut player) = TPlayer::find_by_identity(ctx, identity) {
            player.remove_identity(identity);
            ctx.db.player().id().update(player);
        }
    }
    fn remove_identity(&mut self, identity: &Identity) {
        self.identities.retain(|i| !i.eq(identity));
    }
    pub fn cleanup(ctx: &ReducerContext) {
        for player in ctx.db.player().iter() {
            if player.identities.is_empty() && player.pass_hash.is_none() {
                ctx.db.player().delete(player);
            }
        }
    }
}

pub trait GetPlayer {
    fn player(&self) -> Result<TPlayer, String>;
}

impl GetPlayer for ReducerContext {
    fn player(&self) -> Result<TPlayer, String> {
        TPlayer::find_by_identity(self, &self.sender)
    }
}

#[spacetimedb::reducer]
fn admin_set_temp_pass(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    ctx.is_admin()?;
    let player = ctx
        .db
        .player()
        .id()
        .find(id)
        .context_str("Player not found")?;
    let pass: String = ctx
        .rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    spacetimedb::println!("Temp pass: {id} {} {pass}", player.name);
    let pass_hash = Some(TPlayer::hash_pass(ctx, pass)?);
    ctx.db.player().id().update(TPlayer {
        pass_hash,
        ..player
    });

    Ok(())
}
