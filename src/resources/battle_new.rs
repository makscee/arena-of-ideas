use egui::lerp;

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
        bs.send_event(Event::BattleStart);
        Window::new("Battle", move |ui, _| {
            ui.set_min_size(egui::vec2(800.0, 400.0));
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
    fn send_event(&mut self, event: Event) {
        for f in self
            .world
            .query_filtered::<&Fusion, Without<Corpse>>()
            .iter(&self.world)
        {
            let mut context = Context::new_world(&self.world).take();
            f.react(&event, &mut context).unwrap();
        }
    }
    fn show_slot(&self, i: usize, side: bool, slots: usize, ui: &mut Ui) -> Response {
        let full_rect = ui.available_rect_before_wrap();
        let rect = slot_rect(i, side, full_rect, slots);
        ui.expand_to_include_rect(rect);
        let mut cui = ui.child_ui(rect, *ui.layout(), None);
        let r = cui.allocate_rect(rect, Sense::hover());
        let mut stroke = if r.hovered() {
            STROKE_YELLOW
        } else {
            STROKE_DARK
        };
        let t = cui
            .ctx()
            .animate_bool(Id::new("slot_hovered").with(i).with(side), r.hovered());
        let length = lerp(15.0..=20.0, t);
        stroke.width += t;
        corners_rounded_rect(r.rect.shrink(3.0), length, stroke, ui);
        r
    }
    pub fn show(&mut self, ui: &mut Ui) {
        let slots = global_settings().team_slots as usize;
        let center_rect = slot_rect(0, true, ui.available_rect_before_wrap(), slots);
        let up = center_rect.width() * 0.5;
        for (slot, side) in (1..=slots).cartesian_product([true, false]) {
            self.show_slot(slot, side, slots, ui);
        }
        let unit_size = center_rect.width() * UNIT_SIZE;

        return;
        let rect = Rect::NOTHING;
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

fn slot_rect(i: usize, side: bool, full_rect: Rect, team_slots: usize) -> Rect {
    let total_slots = team_slots * 2 + 1;
    let pos_i = if side {
        (team_slots - i) as i32
    } else {
        (team_slots + i) as i32
    } as f32;
    let size = (full_rect.width() / total_slots as f32).at_most(full_rect.height());
    let mut rect = full_rect;
    rect.set_height(size);
    rect.set_width(size);
    rect.translate(egui::vec2(size * pos_i, 0.0))
}
