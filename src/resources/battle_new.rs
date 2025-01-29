use super::*;

pub struct Battle {
    pub left: Team,
    pub right: Team,
}

pub struct BattleSimulation {
    world: World,
    fusions_left: Vec<Entity>,
    fusions_right: Vec<Entity>,
}

impl Battle {
    pub fn open_window(self, world: &mut World) {
        let mut bs = BattleSimulation::new(self).unwrap();
        Window::new("Battle", move |ui, _| {
            ui.set_min_size(egui::vec2(400.0, 400.0));
            bs.show(ui);
        })
        .push(world);
    }
}

impl BattleSimulation {
    pub fn new(battle: Battle) -> Result<Self, ExpressionError> {
        let mut world = World::new();
        let team_left = world.spawn_empty().id();
        let team_right = world.spawn_empty().id();
        battle.left.unpack(team_left, &mut world.commands());
        battle.right.unpack(team_right, &mut world.commands());
        world.flush();
        for fusion in world.query::<&Fusion>().iter(&world).cloned().collect_vec() {
            fusion.init(&mut world)?;
        }
        fn entities_by_slot(parent: Entity, world: &World) -> Vec<Entity> {
            Context::new_world(&world)
                .children_components_recursive::<UnitSlot>(parent)
                .into_iter()
                .sorted_by_key(|(_, s)| s.slot)
                .map(|(e, _)| e)
                .collect_vec()
        }
        let fusions_left = entities_by_slot(team_left, &world);
        let fusions_right = entities_by_slot(team_right, &world);
        Ok(Self {
            world,
            fusions_left,
            fusions_right,
        })
    }
    pub fn show(&mut self, ui: &mut Ui) {
        let rect = ui.available_rect_before_wrap();
        let mut entities: VecDeque<Entity> = self
            .world
            .query_filtered::<Entity, Without<Parent>>()
            .iter(&self.world)
            .collect();
        let context = Context::new_world(&self.world).take();
        while let Some(entity) = entities.pop_front() {
            let context = context.clone().set_owner(entity).take();
            if context.get_bool(VarName::visible).unwrap_or(true) {
                entities.extend(context.get_children(entity));
                if let Some(rep) = self.world.get::<Representation>(entity) {
                    match RepresentationPlugin::paint_rect(rect, &context, &rep.material, ui) {
                        Ok(_) => {}
                        Err(e) => error!("Rep paint error: {e}"),
                    }
                }
            }
        }
        for fusion in &self.fusions_left {
            let fusion = self.world.get::<Fusion>(*fusion).unwrap();
            fusion.paint(rect, ui, &self.world).log();
        }
    }
}
