use super::*;

#[reducer]
fn incubator_push(
    ctx: &ReducerContext,
    nodes: Vec<TNode>,
    link_from: Option<u64>,
) -> Result<(), String> {
    let player = ctx.player()?;
    let mut next_id = ctx.next_id();
    let nodes = NodeKind::parse_and_reassign_ids(&nodes, &mut next_id).to_str_err()?;
    GlobalData::set_next_id(ctx, next_id);
    let incubator_id = All::load(ctx).incubator_load(ctx)?.id;
    let root_id = nodes[0].id;
    let nodes: HashMap<u64, TNode> = HashMap::from_iter(nodes.into_iter().map(|n| (n.id, n)));
    let link_kinds = NodeKind::get_incubator_links();
    let mut new_links: Vec<(u64, u64)> = default();
    for (_, node) in nodes.iter() {
        let id = node.id;
        let parent_id = node.parent;
        let kind = NodeKind::from_str(&node.kind).unwrap();
        let Some(parent_node) = nodes.get(&parent_id) else {
            continue;
        };
        let parent_kind = NodeKind::from_str(&parent_node.kind).unwrap();
        if link_kinds
            .get(&kind)
            .is_some_and(|links| links.contains(&parent_kind))
        {
            let link =
                TIncubatorLinks::new(id, parent_id, parent_kind.to_string(), kind.to_string())
                    .insert(ctx);
            new_links.push((link.from, link.to));
        } else if link_kinds
            .get(&parent_kind)
            .is_some_and(|links| links.contains(&kind))
        {
            let link =
                TIncubatorLinks::new(parent_id, id, kind.to_string(), parent_kind.to_string())
                    .insert(ctx);
            new_links.push((link.from, link.to));
        }
    }
    if let Some(from_id) = link_from {
        new_links.push((from_id, root_id));
    }
    for mut node in nodes.into_values() {
        node.parent = incubator_id;
        ctx.db.incubator_nodes().insert(TIncubator {
            id: node.id,
            owner: player.id,
        });
        ctx.db.nodes_world().insert(node);
    }
    for (from, to) in new_links {
        TIncubatorVotes::vote(ctx, &player, from, to)?;
    }
    Ok(())
}

#[reducer]
fn incubator_delete(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let i_node = ctx
        .db
        .incubator_nodes()
        .id()
        .find(id)
        .to_e_s_fn(|| format!("Incubator node#{id} not found"))?;
    if i_node.owner != player.id {
        return Err(format!(
            "Incubator node#{id} is not owned by player#{}",
            player.id
        ));
    }
    ctx.db.incubator_nodes().id().delete(id);
    ctx.db.incubator_votes().from().delete(id);
    ctx.db.incubator_votes().to().delete(id);
    ctx.db.incubator_links().from().delete(id);
    ctx.db.incubator_links().to().delete(id);
    ctx.db.nodes_world().id().delete(id);
    Ok(())
}

#[reducer]
fn incubator_vote(ctx: &ReducerContext, from: u64, to: u64) -> Result<(), String> {
    let player = ctx.player()?;
    TIncubatorVotes::vote(ctx, &player, from, to)
}

#[table(public, name = incubator_nodes)]
pub struct TIncubator {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
}

#[table(public, name = incubator_links)]
pub struct TIncubatorLinks {
    #[primary_key]
    pub key: String,
    #[index(btree)]
    pub from: u64,
    #[index(btree)]
    pub to: u64,
    #[index(btree)]
    pub from_kind: String,
    #[index(btree)]
    pub to_kind: String,
    pub score: i64,
}

#[table(public, name = incubator_votes)]
pub struct TIncubatorVotes {
    #[primary_key]
    pub key: String,
    #[index(btree)]
    pub owner: u64,
    #[index(btree)]
    pub from: u64,
    #[index(btree)]
    pub to: u64,
    #[index(btree)]
    pub to_kind: String,
}

impl TIncubatorLinks {
    fn key(from: u64, to: u64) -> String {
        format!("{from}_{to}")
    }
    fn new(from: u64, to: u64, to_kind: String, from_kind: String) -> Self {
        Self {
            key: Self::key(from, to),
            from,
            to,
            to_kind,
            from_kind,
            score: 0,
        }
    }
    fn insert(self, ctx: &ReducerContext) -> Self {
        ctx.db.incubator_links().insert(self)
    }
    fn vote(ctx: &ReducerContext, from: u64, to: u64, delta: i64) -> Result<(), String> {
        let key = Self::key(from, to);
        let mut row = if let Some(row) = ctx.db.incubator_links().key().find(key.clone()) {
            row
        } else {
            let to_kind = TNode::find(ctx, to)
                .to_e_s_fn(|| format!("Failed to find node to#{to}"))?
                .kind;
            let from_kind = TNode::find(ctx, from)
                .to_e_s_fn(|| format!("Failed to find node to#{to}"))?
                .kind;
            ctx.db
                .incubator_links()
                .insert(Self::new(from, to, to_kind, from_kind))
        };
        row.score += delta;
        ctx.db.incubator_links().key().update(row);
        Ok(())
    }
}

impl TIncubatorVotes {
    fn key(owner: u64, from: u64, kind: NodeKind) -> String {
        format!("{owner}_{from}_{kind}")
    }
    fn new(player: &Player, from: u64, to: u64, to_kind: NodeKind) -> Self {
        Self {
            key: Self::key(player.id, from, to_kind),
            owner: player.id,
            from,
            to,
            to_kind: to_kind.to_string(),
        }
    }
    fn vote(ctx: &ReducerContext, player: &Player, from: u64, to: u64) -> Result<(), String> {
        let kind = TNode::find(ctx, to)
            .to_e_s_fn(|| format!("Node {to} not found"))?
            .kind
            .to_kind();
        let key = Self::key(player.id, from, kind);
        if let Some(mut row) = ctx.db.incubator_votes().key().find(key.clone()) {
            TIncubatorLinks::vote(ctx, row.from, row.to, -1)?;
            row.to = to;
            TIncubatorLinks::vote(ctx, row.from, row.to, 1)?;
            ctx.db.incubator_votes().key().update(row);
        } else {
            let row = Self::new(player, from, to, kind);
            TIncubatorLinks::vote(ctx, row.from, row.to, 1)?;
            ctx.db.incubator_votes().insert(row);
        };
        Ok(())
    }
}
