use super::*;

#[table(public, name = nodes)]
pub struct Nodes {
    #[primary_key]
    pub key: String,
    pub id: u64,
    pub kind: String,
    pub parent: u64,
    pub data: String,
}

impl Nodes {
    fn new(id: u64, kind: NodeKind, data: String) -> Self {
        Self {
            key: kind.key(id),
            id,
            kind: kind.to_string(),
            parent: 0,
            data,
        }
    }
}

trait KeyFromKind {
    fn key_next(self, ctx: &ReducerContext) -> String;
    fn key(self, id: u64) -> String;
}
impl KeyFromKind for NodeKind {
    fn key_next(self, ctx: &ReducerContext) -> String {
        format!("{}_{self}", next_id(ctx))
    }
    fn key(self, id: u64) -> String {
        format!("{id}_{self}")
    }
}

trait NodeExt {
    fn insert(&self, ctx: &ReducerContext, id: u64);
    fn update(&self, ctx: &ReducerContext, id: u64);
    fn to_node(&self, id: u64) -> Nodes;
}

impl<T> NodeExt for T
where
    T: Node + GetNodeKind,
{
    fn insert(&self, ctx: &ReducerContext, id: u64) {
        ctx.db
            .nodes()
            .insert(Nodes::new(id, self.kind(), self.get_data()));
    }
    fn update(&self, ctx: &ReducerContext, id: u64) {
        ctx.db.nodes().key().update(self.to_node(id));
    }
    fn to_node(&self, id: u64) -> Nodes {
        Nodes::new(id, self.kind(), self.get_data())
    }
}

#[reducer]
fn node_spawn(
    ctx: &ReducerContext,
    id: Option<u64>,
    kinds: Vec<String>,
    datas: Vec<String>,
) -> Result<(), String> {
    let id = id.unwrap_or_else(|| next_id(ctx));
    for (kind, data) in kinds.into_iter().zip(datas.into_iter()) {
        let kind = NodeKind::from_str(&kind).map_err(|e| e.to_string())?;
        ctx.db.nodes().insert(Nodes::new(id, kind, data));
    }
    Ok(())
}

#[reducer]
fn node_spawn_hero(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let id = next_id(ctx);
    let hero = Hero::new(name);
    let mover = Mover::new();
    hero.insert(ctx, id);
    mover.insert(ctx, id);
    Ok(())
}

#[reducer]
fn node_move(ctx: &ReducerContext, id: u64, x: f32, y: f32) -> Result<(), String> {
    let key = NodeKind::Mover.key(id);
    let data = ctx
        .db
        .nodes()
        .key()
        .find(&key)
        .to_e_s("Mover node not found")?
        .data;
    let mut mover = Mover::default();
    mover.inject_data(&data);
    mover.from = mover.pos(GlobalSettings::get(ctx).hero_speed);
    mover.start_ts = now_seconds();
    mover.target = vec2(x, y);
    mover.update(ctx, id);
    Ok(())
}
