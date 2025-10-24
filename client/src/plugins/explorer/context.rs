use super::*;
use crate::stdb::RemoteTables;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbLinkStrategy {
    /// Only take links where solid = true
    Solid,
    /// Take links with the highest rating (and solid=true as tiebreaker)
    TopRating,
    /// Take links selected by the current player
    PlayerSelection,
}

pub struct DbSource<'a> {
    db: &'a RemoteTables,
    link_strategy: DbLinkStrategy,
}

impl<'a> DbSource<'a> {
    pub fn new(db: &'a RemoteTables) -> Self {
        Self {
            db,
            link_strategy: DbLinkStrategy::Solid,
        }
    }

    pub fn with_strategy(db: &'a RemoteTables, strategy: DbLinkStrategy) -> Self {
        Self {
            db,
            link_strategy: strategy,
        }
    }

    fn filter_links(&self, links: Vec<TNodeLink>, kind: NodeKind) -> Vec<u64> {
        match self.link_strategy {
            DbLinkStrategy::Solid => links
                .into_iter()
                .filter(|l| l.solid)
                .map(|l| l.child)
                .collect(),

            DbLinkStrategy::TopRating => {
                let mut filtered: Vec<TNodeLink> = links.into_iter().collect();
                // Sort by rating descending (highest first), then solid, then by id for consistency
                filtered.sort_by(|a, b| {
                    b.rating
                        .cmp(&a.rating)
                        .then(b.solid.cmp(&a.solid))
                        .then(a.id.cmp(&b.id))
                });

                // Return all children sorted by rating (highest first)
                filtered.into_iter().map(|l| l.child).collect()
            }

            DbLinkStrategy::PlayerSelection => {
                let player_id = player_id();
                // Find links selected by this player
                let player_selections = self
                    .db
                    .player_link_selections()
                    .iter()
                    .filter(|s| s.player_id == player_id)
                    .map(|s| s.selected_link_id)
                    .collect::<std::collections::HashSet<_>>();

                let mut selected_links: Vec<TNodeLink> = links
                    .into_iter()
                    .filter(|l| player_selections.contains(&l.id))
                    .collect();

                // Sort by rating descending for consistency
                selected_links.sort_by(|a, b| {
                    b.rating
                        .cmp(&a.rating)
                        .then(b.solid.cmp(&a.solid))
                        .then(a.id.cmp(&b.id))
                });

                selected_links.into_iter().map(|l| l.child).collect()
            }
        }
    }
}

impl WorldContextExt for RemoteTables {
    fn with_context<R, F>(&self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Context<ClientSource<'_>>) -> NodeResult<R>,
    {
        let source = ClientSource::new_db(self, DbLinkStrategy::Solid);
        Context::exec(source, f)
    }

    fn as_context(&self) -> Context<ClientSource<'_>> {
        Context::new(ClientSource::Db(Box::new(DbSource::new(self))))
    }

    fn with_context_mut<R, F>(&mut self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Context<ClientSource<'_>>) -> NodeResult<R>,
    {
        let source = ClientSource::new_db(self, DbLinkStrategy::Solid);
        Context::exec(source, f)
    }

    fn as_context_mut(&mut self) -> Context<ClientSource<'_>> {
        Context::new(ClientSource::Db(Box::new(DbSource::new(self))))
    }
}

pub trait DbContextExt {
    fn with_context_strategy<R, F>(&self, strategy: DbLinkStrategy, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Context<ClientSource<'_>>) -> NodeResult<R>;
}

impl DbContextExt for RemoteTables {
    fn with_context_strategy<R, F>(&self, strategy: DbLinkStrategy, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Context<ClientSource<'_>>) -> NodeResult<R>,
    {
        let source = ClientSource::new_db(self, strategy);
        Context::exec(source, f)
    }
}

impl<'a> ContextSource for DbSource<'a> {
    fn get_node_kind(&self, node_id: u64) -> NodeResult<NodeKind> {
        self.db
            .nodes_world()
            .id()
            .find(&node_id)
            .and_then(|n| NodeKind::try_from(n.kind.as_str()).ok())
            .ok_or_else(|| NodeError::custom("Node not found"))
    }

    fn get_children(&self, from_id: u64) -> NodeResult<Vec<u64>> {
        let links: Vec<TNodeLink> = self
            .db
            .node_links()
            .iter()
            .filter(|l| l.parent == from_id)
            .collect();

        // For generic get_children, we don't have kind info so apply basic filtering
        Ok(match self.link_strategy {
            DbLinkStrategy::Solid => {
                let mut solid_links: Vec<TNodeLink> =
                    links.into_iter().filter(|l| l.solid).collect();
                solid_links.sort_by(|a, b| b.rating.cmp(&a.rating).then(a.id.cmp(&b.id)));
                solid_links.into_iter().map(|l| l.child).collect()
            }
            DbLinkStrategy::TopRating => {
                let mut sorted_links = links;
                sorted_links.sort_by(|a, b| {
                    b.rating
                        .cmp(&a.rating)
                        .then(b.solid.cmp(&a.solid))
                        .then(a.id.cmp(&b.id))
                });
                sorted_links.into_iter().map(|l| l.child).collect()
            }
            DbLinkStrategy::PlayerSelection => {
                let player_id = player_id();
                let player_selections = self
                    .db
                    .player_link_selections()
                    .iter()
                    .filter(|s| s.player_id == player_id)
                    .map(|s| s.selected_link_id)
                    .collect::<std::collections::HashSet<_>>();

                let mut selected_links: Vec<TNodeLink> = links
                    .into_iter()
                    .filter(|l| player_selections.contains(&l.id))
                    .collect();

                selected_links.sort_by(|a, b| b.rating.cmp(&a.rating).then(a.id.cmp(&b.id)));
                selected_links.into_iter().map(|l| l.child).collect()
            }
        })
    }

    fn get_children_of_kind(&self, from_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        let kind_str = kind.as_ref();
        let links: Vec<TNodeLink> = self
            .db
            .node_links()
            .iter()
            .filter(|l| l.parent == from_id && l.child_kind == kind_str)
            .collect();

        Ok(self.filter_links(links, kind))
    }

    fn get_parents(&self, to_id: u64) -> NodeResult<Vec<u64>> {
        let mut links: Vec<TNodeLink> = self
            .db
            .node_links()
            .iter()
            .filter(|l| l.child == to_id)
            .collect();

        // Sort parents by rating descending for consistency
        links.sort_by(|a, b| {
            b.rating
                .cmp(&a.rating)
                .then(b.solid.cmp(&a.solid))
                .then(a.id.cmp(&b.id))
        });

        Ok(links.into_iter().map(|l| l.parent).collect())
    }

    fn get_parents_of_kind(&self, to_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        let kind_str = kind.as_ref();
        let mut links: Vec<TNodeLink> = self
            .db
            .node_links()
            .iter()
            .filter(|l| l.child == to_id && l.parent_kind == kind_str)
            .collect();

        // Sort parents by rating descending for consistency
        links.sort_by(|a, b| {
            b.rating
                .cmp(&a.rating)
                .then(b.solid.cmp(&a.solid))
                .then(a.id.cmp(&b.id))
        });

        Ok(links.into_iter().map(|l| l.parent).collect())
    }

    fn add_link(&mut self, _from_id: u64, _to_id: u64) -> NodeResult<()> {
        Err(NodeError::custom("Cannot modify DB source"))
    }

    fn remove_link(&mut self, _from_id: u64, _to_id: u64) -> NodeResult<()> {
        Err(NodeError::custom("Cannot modify DB source"))
    }

    fn is_linked(&self, from_id: u64, to_id: u64) -> NodeResult<bool> {
        Ok(self.db.node_links().iter().any(|l| {
            (l.parent == from_id && l.child == to_id) || (l.child == from_id && l.parent == to_id)
        }))
    }

    fn insert_node(
        &mut self,
        _id: u64,
        _owner: u64,
        _kind: NodeKind,
        _data: String,
    ) -> NodeResult<()> {
        Err(NodeError::custom("Cannot modify DB source"))
    }

    fn delete_node(&mut self, _id: u64) -> NodeResult<()> {
        Err(NodeError::custom("Cannot modify DB source"))
    }

    fn get_var_direct(&self, id: u64, var: VarName) -> NodeResult<VarValue> {
        let tnode = self
            .db
            .nodes_world()
            .id()
            .find(&id)
            .ok_or_else(|| NodeError::custom("Node not found"))?;

        let kind = NodeKind::try_from(tnode.kind.as_str())
            .map_err(|_| NodeError::custom("Invalid node kind"))?;

        node_kind_match!(kind, {
            let node = tnode.to_node::<NodeType>()?;
            node.get_var(var)
        })
    }

    fn set_var(&mut self, _id: u64, _var: VarName, _value: VarValue) -> NodeResult<()> {
        Err(NodeError::custom("Cannot modify DB source"))
    }
}
