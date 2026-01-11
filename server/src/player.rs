use super::*;

use schema::NodeKind;

#[reducer]
fn register(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let name = NPlayer::validate_name(ctx, name)?;
    NPlayer::clear_identity(ctx, &ctx.rctx().sender);
    let mut player = NPlayer::new(ctx.next_id(), ID_PLAYERS, name);
    player
        .player_data
        .set_loaded(NPlayerData::new(ctx.next_id(), ID_PLAYERS, false, 0));
    player.identity.set_loaded(NPlayerIdentity::new(
        ctx.next_id(),
        ID_PLAYERS,
        Some(ctx.rctx().sender.to_string()),
    ));
    ctx.source_mut().commit(player)?;
    Ok(())
}

#[reducer]
fn login_by_identity(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let player = ctx.player().track()?.login(ctx).track()?;
    ctx.source_mut().commit(player).track()?;
    Ok(())
}

#[reducer]
fn logout(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let player = ctx.player()?.logout(ctx)?;
    player.identity.load_node(ctx)?.delete(ctx);
    Ok(())
}

#[reducer(client_disconnected)]
fn identity_disconnected(ctx: &ReducerContext) {
    let ctx = &mut ctx.as_context();
    if let Ok(player) = ctx.player() {
        if let Ok(logged_out) = player.logout(ctx) {
            ctx.source_mut().commit(logged_out).log();
        }
    }
}

impl NPlayer {
    fn validate_name(ctx: &ServerContext, name: String) -> Result<String, String> {
        let name = name.to_lowercase();
        if name.is_empty() {
            Err("Names must not be empty".to_string())
        } else if let Some(c) = name.chars().find(|c| !c.is_alphanumeric()) {
            Err(format!("Wrong character: {c}"))
        } else if NPlayer::find_by_data(ctx, &NPlayer::new(0, 0, name.clone()).get_data()).is_some()
        {
            Err(format!("Name is taken"))
        } else {
            Ok(name)
        }
    }
    pub fn find_identity(ctx: &ServerContext, identity: &Identity) -> Option<NPlayerIdentity> {
        NPlayerIdentity::find_by_data(ctx, &ron::to_string(&Some(identity.to_string())).unwrap())
            .into_iter()
            .next()
    }
    fn login(mut self, ctx: &ServerContext) -> NodeResult<Self> {
        let ts = ctx.rctx().timestamp.to_micros_since_unix_epoch() as u64;
        let data = self.player_data.load_node_mut(ctx)?;
        data.last_login = ts;
        data.online = true;
        Ok(self)
    }
    fn logout(mut self, ctx: &ServerContext) -> Result<Self, String> {
        let data = self.player_data.load_node_mut(ctx)?;
        data.online = false;
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
            .ok_or_else(|| NodeError::custom("NPlayerIdentity not found"))
            .track()?;
        let id = self
            .get_parents_of_kind(identity.id, NodeKind::NPlayer)?
            .into_iter()
            .next()
            .to_not_found()
            .track()?;
        NPlayer::load(self.source(), id)
    }
}
