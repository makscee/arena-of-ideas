use super::*;

pub struct Context<'a> {
    pub rc: &'a ReducerContext,
    pub player: TPlayer,
}

impl<'a> Context<'a> {
    pub fn empty(ctx: &'a ReducerContext) -> Self {
        Self {
            rc: ctx,
            player: TPlayer::empty(),
        }
    }
    pub fn new(ctx: &'a ReducerContext) -> Result<Self, String> {
        let player = ctx.player()?;
        Ok(Self { rc: ctx, player })
    }
    pub fn pid(&self) -> u64 {
        self.player.id
    }
    pub fn next_id(&self) -> u64 {
        next_id(self.rc)
    }
    pub fn global_settings(&self) -> GlobalSettings {
        GlobalSettings::get(self.rc)
    }
}

pub trait RcExt {
    fn wrap(&self) -> Result<Context, String>;
}

impl RcExt for ReducerContext {
    fn wrap(&self) -> Result<Context, String> {
        Context::new(self)
    }
}
