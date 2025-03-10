use super::*;

#[reducer]
fn incubator_push(ctx: &ReducerContext, kind: String, datas: Vec<String>) -> Result<(), String> {
    let player = ctx.player()?;
    let kind = NodeKind::from_str(&kind).map_err(|e| e.to_string())?;
    let parent = All::load(ctx).incubator_load(ctx)?.id;
    let nodes = kind.tnode_vec_from_strings(ctx, &datas).to_str_err()?;
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
            let link = TIncubatorLinks::new(id, parent, parent_kind.to_string()).insert(ctx);
            new_links.push((link.from, link.to));
        } else if link_kinds
            .get(&parent_kind)
            .is_some_and(|links| links.contains(&kind))
        {
            let link = TIncubatorLinks::new(parent_id, id, kind.to_string()).insert(ctx);
            new_links.push((link.from, link.to));
        }
    }
    for mut node in nodes.into_values() {
        node.parent = parent;
        ctx.db.nodes_world().insert(node);
    }
    ctx.db.incubator_nodes().insert(TIncubator {
        id: ctx.next_id(),
        owner: player.id,
    });
    for (from, to) in new_links {
        TIncubatorVotes::vote(ctx, &player, from, to, 1)?;
    }
    Ok(())
}

#[reducer]
fn incubator_vote(ctx: &ReducerContext, from: u64, to: u64, vote: i8) -> Result<(), String> {
    let player = ctx.player()?;
    TIncubatorVotes::vote(ctx, &player, from, to, vote)
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
    pub target: String,
    pub vote: i8,
}

impl TIncubatorLinks {
    fn key(from: u64, to: u64) -> String {
        format!("{from}_{to}")
    }
    fn new(from: u64, to: u64, to_kind: String) -> Self {
        Self {
            key: Self::key(from, to),
            from,
            to,
            to_kind,
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
            let kind = ctx
                .db
                .nodes_world()
                .id()
                .find(to)
                .to_e_s_fn(|| format!("Failed to find node to#{to}"))?
                .kind;
            ctx.db.incubator_links().insert(Self::new(from, to, kind))
        };
        row.score += delta;
        ctx.db.incubator_links().key().update(row);
        Ok(())
    }
}

impl TIncubatorVotes {
    fn key(owner: u64, target: &str) -> String {
        format!("{owner}_{target}")
    }
    fn new(player: &Player, target: String, vote: i8) -> Self {
        Self {
            key: Self::key(player.id, &target),
            owner: player.id,
            target,
            vote,
        }
    }
    fn vote(
        ctx: &ReducerContext,
        player: &Player,
        from: u64,
        to: u64,
        vote: i8,
    ) -> Result<(), String> {
        let target = TIncubatorLinks::key(from, to);
        let key = Self::key(player.id, &target);
        let delta = if let Some(mut row) = ctx.db.incubator_votes().key().find(key.clone()) {
            let delta = vote - row.vote;
            row.vote = vote;
            ctx.db.incubator_votes().key().update(row);
            delta
        } else {
            ctx.db
                .incubator_votes()
                .insert(Self::new(player, target.clone(), vote));
            vote
        };
        TIncubatorLinks::vote(ctx, from, to, delta as i64)
    }
}
