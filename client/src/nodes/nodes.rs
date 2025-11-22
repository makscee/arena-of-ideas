use super::*;

use spacetimedb_sats::serde::SerdeWrapper;
use std::fmt::Debug;

include!(concat!(env!("OUT_DIR"), "/client_nodes.rs"));

pub trait ClientNode:
    Default + BevyComponent + Sized + FDisplay + Debug + StringData + Clone + ToCstr + schema::Node
{
    fn spawn(self, ctx: &mut ClientContext, entity: Option<Entity>) -> NodeResult<()>;
    fn save(self, ctx: &mut ClientContext) -> NodeResult<()>;
    fn load_components(&mut self, ctx: &ClientContext) -> NodeResult<&mut Self>;
    fn load_all(&mut self, ctx: &ClientContext) -> NodeResult<&mut Self>;
    fn entity(&self, ctx: &ClientContext) -> NodeResult<Entity> {
        ctx.entity(self.id())
    }
    fn from_file(path: &str) -> NodeResult<Self> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| NodeError::from(format!("Failed to read file {}: {}", path, e)))?;
        let mut node = Self::default();
        node.inject_data(&data)?;
        Ok(node)
    }
    fn remap_ids(mut self) -> Self {
        let mut next_id = next_id();
        let mut id_map = std::collections::HashMap::new();
        self.reassign_ids(&mut next_id, &mut id_map);
        set_next_id(next_id);
        self.set_dirty(true);
        self
    }
    fn to_tnode(&self) -> TNode {
        TNode {
            id: self.id(),
            owner: self.owner(),
            kind: self.kind().to_string(),
            data: self.get_data(),
            rating: 0,
        }
    }
    fn rating(&self) -> i32 {
        self.id().node_rating().unwrap_or_default()
    }
}

pub trait NodeExt: ClientNode {
    fn db_load(id: u64) -> NodeResult<Self> {
        TNode::find(id).to_not_found()?.to_node()
    }
}

impl<T: ClientNode> NodeExt for T {}

impl TNode {
    pub fn find(id: u64) -> Option<Self> {
        cn().db.nodes_world().id().find(&id)
    }
    pub fn kind(&self) -> NodeKind {
        self.kind.to_kind()
    }
    pub fn to_node<T: ClientNode + StringData>(&self) -> NodeResult<T> {
        let mut d = T::default();
        d.inject_data(&self.data)?;
        d.set_id(self.id);
        d.set_owner(self.owner);
        Ok(d)
    }
    pub fn to_ron(self) -> String {
        ron::to_string(&SerdeWrapper::new(self)).unwrap()
    }
}

pub trait NodeKindOnSpawn {
    fn on_spawn(self, context: &mut ClientContext, id: u64) -> NodeResult<()>;
}

impl NodeKindOnSpawn for NodeKind {
    fn on_spawn(self, ctx: &mut ClientContext, id: u64) -> NodeResult<()> {
        debug!("on spawn {self} {id} ");
        let entity = ctx.entity(id).track()?;
        let vars = node_kind_match!(self, ctx.load::<NodeType>(id).track()?.get_vars());

        // Only create NodeStateHistory for battle simulations
        if ctx.battle().is_ok() {
            let world = ctx.world_mut()?;
            let mut emut = world.entity_mut(entity);
            let mut ns = if let Some(ns) = emut.get_mut::<NodeStateHistory>() {
                ns
            } else {
                emut.insert(NodeStateHistory::default())
                    .get_mut::<NodeStateHistory>()
                    .unwrap()
            };
            ns.init_vars(vars.into_iter());
        }

        let world = ctx.world_mut()?;
        let mut emut = world.entity_mut(entity);
        if let Some(mut ne) = emut.get_mut::<NodeEntityComponent>() {
            ne.add_node(id, self);
        } else {
            emut.insert(NodeEntityComponent::new(id, self));
        };

        emut.insert((Transform::default(), Visibility::default()));

        match self {
            NodeKind::NUnit => {
                let unit = ctx.load::<NUnit>(id)?;
                if let Ok(mut rep) = unit.representation_ref(ctx).cloned() {
                    rep.material.0.append(&mut unit_rep().material.0.clone());
                    rep.spawn(ctx, Some(entity))?;
                } else {
                    let rep_id = next_id();
                    unit_rep()
                        .clone()
                        .with_id(rep_id)
                        .spawn(ctx, Some(entity))?;
                    ctx.add_link(id, rep_id)?;
                }
            }
            NodeKind::NStatusMagic => {
                if ctx
                    .get_children_of_kind(id, NodeKind::NStatusRepresentation)?
                    .is_empty()
                {
                    let rep_id = next_id();
                    status_rep()
                        .clone()
                        .with_id(rep_id)
                        .spawn(ctx, Some(entity))?;
                    ctx.add_link(id, rep_id)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl NHouse {
    pub fn color_for_text(&self, ctx: &ClientContext) -> Color32 {
        self.color_ref(ctx)
            .map(|c| c.color.c32())
            .unwrap_or_else(|_| colorix().low_contrast_text())
    }
}

impl NUnit {
    pub fn show_status_tags(
        &self,
        rect: Rect,
        ctx: &mut ClientContext,
        ui: &mut Ui,
    ) -> NodeResult<()> {
        let ui = &mut ui.new_child(
            UiBuilder::new()
                .max_rect(
                    Rect::from_center_size(rect.center_bottom(), egui::vec2(rect.width(), 0.0))
                        .translate(egui::vec2(0.0, 15.0)),
                )
                .layout(Layout::left_to_right(Align::Center).with_main_wrap(true)),
        );
        for status in ctx.load_children_ref::<NStatusMagic>(self.id)? {
            if !ctx
                .get_var_inherited(status.id, VarName::visible)
                .get_bool()?
            {
                continue;
            }
            let color = ctx
                .get_var_inherited(status.id, VarName::color)
                .get_color()?;
            let x = ctx.get_var_inherited(status.id, VarName::stax).get_i32()?;
            if x > 0 {
                TagWidget::new_name_value(status.name().to_string().cut_start(2), color, x)
                    .ui(ui)
                    .on_hover_ui(|ui| {
                        ctx.exec_ref(|ctx| {
                            ctx.with_owner(status.id, |ctx| {
                                status.render_card(ctx, ui);
                                Ok(())
                            })
                        })
                        .ui(ui);
                    });
            }
        }
        Ok(())
    }
}
