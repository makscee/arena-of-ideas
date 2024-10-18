use super::*;

#[derive(Asset, Deserialize, Serialize, Debug, Clone, TypePath, Default, PartialEq)]
pub struct Vfx {
    pub anim: Anim,
    pub representation: Representation,
    #[serde(default)]
    pub state: VarState,
    #[serde(default)]
    pub timeframe: f32,
    #[serde(default)]
    pub duration: Option<f32>,
    pub parent: Option<Entity>,
}

impl Vfx {
    pub fn get(name: &str) -> Self {
        game_assets()
            .vfxs
            .get(name)
            .with_context(|| format!("Vfx {name} not loaded"))
            .unwrap()
            .clone()
    }
    pub fn unpack(self, world: &mut World) -> Result<Entity> {
        let entity = world.spawn_empty().id();
        self.representation.unpack(entity, world);
        if let Some(parent) = self.parent {
            world.entity_mut(entity).set_parent(parent);
        }
        self.state.attach(entity, 0, world);
        self.anim.apply(Context::new(entity), world)?;
        if let Some(duration) = self.duration {
            let mut state = VarState::get_mut(entity, world);
            state.init(VarName::Visible, true.into());
            state.push_change(
                VarName::Visible,
                default(),
                VarChange {
                    t: duration,
                    ..default()
                },
            );
        }
        if self.timeframe > 0.0 {
            gt().advance_insert(self.timeframe);
        }
        Ok(entity)
    }
    pub fn set_var(mut self, var: VarName, value: VarValue) -> Self {
        self.state.init(var, value);
        self
    }
    pub fn set_parent(mut self, parent: Entity) -> Self {
        self.parent = Some(parent);
        self
    }
    pub fn attach_context(mut self, context: &Context, world: &World) -> Self {
        if let Ok(owner_pos) = UnitPlugin::get_unit_position(context.owner(), world) {
            if let Ok(target_pos) = context
                .get_target()
                .and_then(|t| UnitPlugin::get_unit_position(t, world))
            {
                let delta = target_pos - owner_pos;
                self = self.set_var(VarName::Delta, VarValue::Vec2(delta));
            }
        }
        self = self.set_parent(context.owner());
        for (var, value) in context.get_all_vars() {
            self = self.set_var(var, value);
        }
        self
    }
}

impl ShowEditor for Vfx {
    fn transparent() -> bool {
        true
    }
    fn show_content(&mut self, context: &Context, world: &mut World, ui: &mut Ui) {
        show_collapsing_node("anim", &mut self.anim, context, ui, world);
        show_collapsing_node(
            "representation",
            &mut self.representation,
            context,
            ui,
            world,
        );
        let mut c = self.duration.is_some();
        ui.horizontal(|ui| {
            if Checkbox::new(&mut c, "duration").ui(ui).changed() {
                if c {
                    self.duration = Some(1.0);
                } else {
                    self.duration = None;
                }
            }
            if let Some(duration) = self.duration.as_mut() {
                DragValue::new(duration).ui(ui);
            }
        });
        DragValue::new(&mut self.timeframe)
            .prefix("timeframe: ")
            .ui(ui);
    }
    fn get_variants() -> impl Iterator<Item = Self> {
        None.into_iter()
    }
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        default()
    }
    fn show_children(&mut self, _context: &Context, _world: &mut World, _ui: &mut Ui) {}
}

impl ToCstr for Vfx {
    fn cstr(&self) -> Cstr {
        self.anim.cstr()
    }
}
