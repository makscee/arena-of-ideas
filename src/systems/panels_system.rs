use super::*;

pub struct PanelsSystem {}
const PADDING: f32 = 0.05;

impl System for PanelsSystem {
    fn update(&mut self, _: &mut legion::World, resources: &mut Resources) {
        let bot_left_corner = vec2(-resources.camera.aspect_ratio, -1.0) + vec2(PADDING, PADDING);
        let bot_right_corner = vec2(resources.camera.aspect_ratio, -1.0) + vec2(-PADDING, PADDING);
        let top_left_corner =
            vec2(-resources.camera.aspect_ratio, 1.0) + vec2(PADDING, -PADDING * 2.0);

        let mut offset = vec2::ZERO;
        for panel in resources.panels_data.push.iter_mut() {
            panel.need_pos = bot_left_corner + offset;
            if panel.t == 0.0 && panel.state == PanelState::Open {
                panel.shader.middle.parameters.r#box.pos = panel.need_pos;
            }
            offset += vec2(
                0.0,
                PADDING + panel.shader.middle.parameters.r#box.size.y * 2.0,
            );
        }
        let mut offset = vec2::ZERO;
        for panel in resources.panels_data.hint.iter_mut() {
            panel.need_pos = bot_right_corner + offset;
            if panel.t == 0.0 && panel.state == PanelState::Open {
                panel.shader.middle.parameters.r#box.pos = panel.need_pos;
            }
            offset += vec2(
                0.0,
                PADDING + panel.shader.middle.parameters.r#box.size.y * 2.0,
            );
        }

        if let Some(panel) = resources.panels_data.stats.as_mut() {
            panel.need_pos = top_left_corner;
        }

        let delta_time = resources.delta_time;
        let global_time = resources.global_time;
        for panel in resources
            .panels_data
            .push
            .iter_mut()
            .chain(resources.panels_data.stats.iter_mut())
            .chain(resources.panels_data.hint.iter_mut())
            .chain(resources.panels_data.alert.iter_mut())
        {
            panel.update(delta_time, global_time)
        }

        resources.panels_data.alert.retain(|x| !x.is_closed());
        resources.panels_data.push.retain(|x| !x.is_closed());
        resources.panels_data.hint.retain(|x| !x.is_closed());

        resources.frame_shaders.extend(
            resources
                .panels_data
                .push
                .iter_mut()
                .chain(resources.panels_data.stats.iter_mut())
                .chain(resources.panels_data.alert.iter_mut())
                .chain(resources.panels_data.hint.iter_mut())
                .map(|x| x.shader.clone()),
        );
    }
}

impl PanelsSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn add_text_alert(
        color: Rgba<f32>,
        title: &str,
        text: &str,
        pos: vec2<f32>,
        buttons: Vec<PanelFooterButton>,
        resources: &mut Resources,
    ) {
        Self::add_alert(
            color,
            title,
            Self::generate_text_shader(text, vec2(0.15, 0.05), &resources.options),
            pos,
            buttons,
            resources,
        )
    }

    pub fn add_alert(
        color: Rgba<f32>,
        title: &str,
        body: ShaderChain,
        pos: vec2<f32>,
        buttons: Vec<PanelFooterButton>,
        resources: &mut Resources,
    ) {
        let mut panel = body.wrap_panel_header(title, &resources.options);
        if !buttons.is_empty() {
            panel = panel.wrap_panel_footer(buttons, &resources.options);
        }
        let mut panel = panel.panel(PanelType::Alert, Some(color), resources);
        panel.need_pos = pos;
        panel.shader.insert_int_ref(
            "u_index".to_owned(),
            resources.panels_data.alert.len() as i32,
        );
        resources.panels_data.alert.push(panel);
    }

    pub fn find_alert_mut(entity: legion::Entity, resources: &mut Resources) -> Option<&mut Panel> {
        resources
            .panels_data
            .alert
            .iter_mut()
            .find(|x| x.shader.middle.entity == Some(entity))
    }

    pub fn open_card_choice(choice: CardChoice, resources: &mut Resources) {
        if resources.panels_data.choice_options.is_some() {
            return;
        }
        let padding = resources.options.floats.panel_row_padding;
        let faction = Faction::Team;
        let cards: Vec<(String, ShaderChain, usize, Rgba<f32>, Rgba<f32>)> = match &choice {
            CardChoice::ShopOffers { units, buffs } => {
                let mut result: Vec<(String, ShaderChain, usize, Rgba<f32>, Rgba<f32>)> = default();
                let body_color = resources.options.colors.light;
                for unit in units {
                    let rarity = HeroPool::rarity_by_name(&unit.name, resources);
                    let name = format!("{} Hero", rarity);
                    let shader = unit.get_ui_shader(faction, true, resources);
                    let panel_color = rarity.color(resources);
                    result.push((name, shader, 3, panel_color, body_color));
                }
                for (buff, target) in buffs {
                    let name = target.name();
                    let status = StatusLibrary::get(&buff.name, resources);
                    let shader =
                        status.generate_card_shader(&buff.name, Some(buff.charges), resources);
                    let panel_color = buff.rarity.color(resources);
                    result.push((name, shader, target.cost(), panel_color, body_color));
                }
                result
            }
            _ => panic!(),
        };
        let title = "Shop";
        let panel_color = resources.options.colors.shop;
        let shaders = cards
            .into_iter()
            .enumerate()
            .map(|(ind, (name, shader, cost, panel_color, body_color))| {
                let mut shader = Self::generate_card_panel(shader, &resources.options)
                    .wrap_panel_header(&name, &resources.options)
                    .wrap_panel_footer(vec![PanelFooterButton::Buy { cost }], &resources.options)
                    .insert_int("u_index".to_owned(), ind as i32)
                    .set_panel_color(panel_color)
                    .set_panel_body_color(body_color);

                fn update_handler(
                    _: HandleEvent,
                    _: legion::Entity,
                    shader: &mut Shader,
                    _: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    if let Some(chosen) = resources.panels_data.chosen_ind {
                        if shader.parameters.uniforms.try_get_int("u_index").unwrap()
                            == chosen as i32
                        {
                            shader.insert_color_ref(
                                "u_panel_body_color".to_owned(),
                                resources.options.colors.active,
                            );
                        }
                    }
                    let index = shader.get_int("u_index") as usize;
                    if resources.panels_data.removed_inds.contains(&index) {
                        shader.set_enabled(false);
                    }
                }
                shader.middle.pre_update_handlers.push(update_handler);
                shader
            })
            .collect_vec();
        let buttons = match choice {
            CardChoice::BuyBuff { .. } | CardChoice::BuyHero { .. } => {
                vec![PanelFooterButton::Reroll, PanelFooterButton::Accept]
            }
            CardChoice::ShopOffers { .. } => vec![PanelFooterButton::Reroll],
        };
        resources.panels_data.choice_options = Some(choice);
        let mut panel =
            ShaderChain::wrap_panel_body_row(shaders, vec2(padding, padding), &resources.options)
                .wrap_panel_header(title, &resources.options)
                .wrap_panel_footer(buttons, &resources.options);
        panel.middle.parameters.r#box.pos.y += 0.3;
        resources.panels_data.alert.push(panel.panel(
            PanelType::Alert,
            Some(panel_color),
            resources,
        ));
    }

    pub fn open_card_list(
        mut shaders: Vec<ShaderChain>,
        title: &str,
        color: Rgba<f32>,
        pos: vec2<f32>,
        row_limit: usize,
        input_handler: Handler,
        button_name: &str,
        resources: &mut Resources,
    ) -> Option<legion::Entity> {
        for shader in shaders.iter_mut() {
            shader
                .middle
                .parameters
                .merge(&resources.options.parameters.panel_card, true);
        }

        let row_limit = if row_limit == 0 || row_limit > shaders.len() {
            shaders.len()
        } else {
            row_limit
        };

        let mut rows: Vec<ShaderChain> = default();
        let padding = resources.options.floats.panel_row_padding;
        while !shaders.is_empty() {
            let limit = row_limit.min(shaders.len());
            let shaders = shaders.drain(0..limit).collect_vec();
            let row = ShaderChain::wrap_panel_body_row(
                shaders,
                vec2(padding, padding),
                &resources.options,
            );

            rows.push(row);
        }
        let shader = ShaderChain::wrap_panel_body_column(
            rows,
            resources.options.floats.panel_column_padding,
            &resources.options,
        );

        let mut panel = shader
            .wrap_panel_header(title, &resources.options)
            .wrap_panel_footer(
                vec![PanelFooterButton::Custom {
                    name: button_name.to_owned(),
                    handler: input_handler,
                }],
                &resources.options,
            )
            .panel(PanelType::Alert, Some(color), resources);
        panel.need_pos = pos;
        let entity = panel.shader.middle.entity;
        resources.panels_data.alert.push(panel);
        entity
    }

    pub fn open_push(color: Rgba<f32>, title: &str, text: &str, resources: &mut Resources) {
        let panel = Self::generate_text_shader(text, vec2::ZERO, &resources.options)
            .wrap_panel_header(title, &resources.options);
        resources
            .panels_data
            .push
            .insert(0, panel.panel(PanelType::Push, Some(color), resources));
    }

    pub fn open_hint(color: Rgba<f32>, title: &str, text: &str, resources: &mut Resources) {
        let panel = Self::generate_text_shader(&text, vec2::ZERO, &resources.options)
            .wrap_panel_header(&title, &resources.options)
            .panel(PanelType::Hint, Some(color), resources);
        resources.panels_data.hint.push(panel);
    }

    pub fn open_stats(world: &legion::World, resources: &mut Resources) {
        let text = Self::get_stats_text(world, resources);
        let panel = Self::generate_text_shader(&text, vec2(0.0, 0.0), &resources.options);
        let panel = panel.wrap_panel_header("Stats", &resources.options).panel(
            PanelType::Stats,
            Some(resources.options.colors.primary),
            resources,
        );
        resources.panels_data.stats = Some(panel);
    }

    pub fn close_stats(resources: &mut Resources) {
        resources.panels_data.stats = None;
    }

    pub fn get_stats_text(world: &legion::World, resources: &mut Resources) -> String {
        let mut texts = Vec::default();
        texts.push(format!("g: {}", ShopSystem::get_g(world)));
        texts.push(format!(
            "level: {}/{}",
            Ladder::current_ind(resources) + 1,
            Ladder::count(resources)
        ));
        texts.push(format!("score: {}", resources.battle_data.total_score));
        texts.join("\n")
    }

    pub fn refresh_stats(world: &legion::World, resources: &mut Resources) {
        if resources.panels_data.stats.is_some() {
            let text = Self::get_stats_text(world, resources);
            let panel = resources.panels_data.stats.as_mut().unwrap();
            panel
                .shader
                .insert_string_ref("u_panel_text".to_owned(), text, 0);
        }
    }

    pub fn close_alert(entity: legion::Entity, resources: &mut Resources) {
        for panel in resources.panels_data.alert.iter_mut() {
            if panel.shader.middle.entity == Some(entity)
                || panel.shader.middle.parent == Some(entity)
            {
                panel.state = PanelState::Closed;
            }
        }
    }

    pub fn close_all_alerts(resources: &mut Resources) {
        for panel in resources.panels_data.alert.iter_mut() {
            panel.state = PanelState::Closed;
        }
    }

    pub fn close_hints(resources: &mut Resources) {
        for panel in resources.panels_data.hint.iter_mut() {
            panel.state = PanelState::Closed;
        }
    }

    pub fn clear(resources: &mut Resources) {
        resources.panels_data.alert.clear();
        resources.panels_data.push.clear();
        resources.panels_data.hint.clear();
        resources.panels_data.removed_inds.clear();
        resources.panels_data.stats = None;
        resources.panels_data.choice_options = None;
        resources.panels_data.chosen_ind = None;
    }

    pub fn generate_text_shader(
        text: &str,
        extra_size: vec2<f32>,
        options: &Options,
    ) -> ShaderChain {
        let mut shader = options.shaders.panel_text.clone().insert_string(
            "u_text".to_owned(),
            text.to_owned(),
            0,
        );
        shader.middle.parameters.r#box.size += extra_size;
        let lines = text.chars().map(|x| (x == '\n') as i32).sum::<i32>() + 1;
        let per_line = shader.middle.parameters.r#box.size.y;
        shader.middle.parameters.r#box.size.y = lines as f32 * per_line;
        let padding = options.floats.panel_text_padding;
        shader = shader.wrap_panel_body(vec2(padding, padding), options);
        shader
    }

    pub fn generate_card_panel(mut card: ShaderChain, options: &Options) -> ShaderChain {
        card.middle
            .parameters
            .merge(&options.parameters.panel_card, true);
        let padding = options.floats.panel_card_padding;
        let card = card.wrap_panel_body(padding, options);
        card
    }
}

#[derive(Default)]
pub struct PanelsData {
    pub alert: Vec<Panel>,
    pub push: Vec<Panel>,
    pub hint: Vec<Panel>,
    pub stats: Option<Panel>,
    pub choice_options: Option<CardChoice>,
    pub chosen_ind: Option<usize>,
    pub removed_inds: HashSet<usize>,
}

#[derive(Debug, Clone)]
pub enum CardChoice {
    BuyHero {
        units: Vec<PackedUnit>,
    },
    BuyBuff {
        buffs: Vec<Buff>,
        target: BuffTarget,
    },
    ShopOffers {
        units: Vec<PackedUnit>,
        buffs: Vec<(Buff, BuffTarget)>,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum BuffTarget {
    Single { slot: Option<usize> },
    Aoe,
}

impl BuffTarget {
    pub fn name(&self) -> String {
        match self {
            BuffTarget::Single { .. } => "Hero Buff".to_owned(),
            BuffTarget::Aoe => "Aoe Buff".to_owned(),
        }
    }

    pub fn random() -> BuffTarget {
        let rand: f32 = (&mut thread_rng()).gen_range(0.0..1.0);
        if rand < 0.8 {
            BuffTarget::Single { slot: None }
        } else {
            BuffTarget::Aoe
        }
    }

    pub fn cost(&self) -> usize {
        match self {
            BuffTarget::Single { .. } => 2,
            BuffTarget::Aoe => 6,
        }
    }
}

impl CardChoice {
    pub fn do_choose(&self, ind: usize, world: &mut legion::World, resources: &mut Resources) {
        match self {
            CardChoice::BuyHero { units } => {
                let unit = units.get(ind).cloned().unwrap();
                ShopSystem::add_unit_to_team(unit, world, resources);
            }
            CardChoice::BuyBuff { buffs, target } => {
                let buff = { buffs.get(ind).cloned().unwrap() };
                ShopSystem::start_buff_apply(
                    buff.name,
                    buff.charges,
                    target.clone(),
                    world,
                    resources,
                )
            }
            CardChoice::ShopOffers { units, buffs } => {
                resources.panels_data.removed_inds.insert(ind);
                if ind < units.len() {
                    let unit = units.get(ind).cloned().unwrap();
                    ShopSystem::add_unit_to_team(unit, world, resources);
                    ShopSystem::change_g(-3, Some("Buy Hero"), world, resources);
                } else if ind - units.len() < buffs.len() {
                    let buff = { buffs.get(ind - units.len()).cloned().unwrap() };
                    let (buff, target) = buff;
                    ShopSystem::start_buff_apply(buff.name, buff.charges, target, world, resources);
                    ShopSystem::change_g(
                        -(target.cost() as i32),
                        Some(&target.name()),
                        world,
                        resources,
                    );
                } else {
                    panic!("Wrong shop offer index: {ind}")
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Panel {
    pub shader: ShaderChain,
    pub need_pos: vec2<f32>,
    pub state: PanelState,
    pub r#type: PanelType,
    pub t: Time,
    pub ts: Time,
}

impl Panel {
    pub fn update(&mut self, delta_time: Time, global_time: Time) {
        const SPEED: f32 = 10.0;

        self.shader.middle.parameters.r#box.pos +=
            (self.need_pos - self.shader.middle.parameters.r#box.pos) * SPEED * delta_time;
        if self.state == PanelState::Open {
            let duration = self.r#type.duration();
            if duration > 0.0 && self.ts + duration < global_time {
                self.state = PanelState::Closed;
            }
        }
        match self.state {
            PanelState::Open => self.t = (self.t + delta_time * 1.5).min(1.0),
            PanelState::Closed => self.t = (self.t - delta_time * 1.5).max(0.0),
        }
        self.shader
            .insert_float_ref("u_open".to_owned(), EasingType::QuartInOut.f(self.t));
    }

    pub fn is_closed(&self) -> bool {
        self.state == PanelState::Closed && self.t <= 0.0
    }
}

impl ShaderChain {
    pub fn wrap_panel_body_row(
        mut shaders: Vec<ShaderChain>,
        padding: vec2<f32>,
        options: &Options,
    ) -> Self {
        let size = {
            let shader = &shaders[0];
            shader.middle.parameters.r#box.size
                * shader
                    .middle
                    .parameters
                    .uniforms
                    .try_get_float("u_scale")
                    .unwrap_or(1.0)
        };
        let mut shader = shaders.remove(0).wrap_panel_body(padding, options);
        let count = shaders.len() as f32;
        shader.middle.parameters.r#box.size.x += count * size.x;
        shader.middle.parameters.r#box.size +=
            vec2(options.floats.panel_row_spacing * (count + 1.0), padding.y);

        shader.after.extend(shaders.drain(..));
        let spacing = 2.0 / shader.after.len() as f32;
        let mut anchor_position = -1.0 + spacing * 0.5;
        for shader in shader.after.iter_mut() {
            shader.middle.parameters.r#box.anchor.x = anchor_position;
            anchor_position += spacing;
        }

        shader
    }
    pub fn wrap_panel_body_column(
        mut shaders: Vec<ShaderChain>,
        padding: f32,
        options: &Options,
    ) -> Self {
        let size = {
            let shader = &shaders[0];
            shader.middle.parameters.r#box.size
                * shader
                    .middle
                    .parameters
                    .uniforms
                    .try_get_float("u_scale")
                    .unwrap_or(1.0)
        };
        let mut shader = shaders
            .remove(0)
            .wrap_panel_body(vec2(padding, padding), options);
        let count = shaders.len() as f32;
        shader.middle.parameters.r#box.size.y += count * size.y;
        shader.middle.parameters.r#box.size +=
            vec2(0.0, options.floats.panel_column_spacing * count);

        shader.after.extend(shaders.drain(..));
        let spacing = 2.0 / shader.after.len() as f32;
        let mut anchor_position = 1.0 - spacing * 0.5;
        for shader in shader.after.iter_mut() {
            shader.middle.parameters.r#box.anchor.y = anchor_position;
            anchor_position -= spacing;
        }

        shader
    }

    pub fn wrap_panel_body(self, padding: vec2<f32>, options: &Options) -> Self {
        let mut shader = options.shaders.panel_body.clone();
        let scale = self
            .middle
            .parameters
            .uniforms
            .try_get_float("u_scale")
            .unwrap_or(1.0);
        shader.middle.parameters.r#box = self.middle.parameters.r#box;
        shader.middle.parameters.r#box.anchor = vec2::ZERO;
        shader.middle.parameters.r#box.center = vec2::ZERO;
        shader.middle.parameters.r#box.size += padding;
        shader.middle.parameters.r#box.size *= scale;
        shader.after.push(self);
        shader.middle.entity = Some(new_entity());
        shader
    }

    pub fn wrap_panel_header(mut self, title: &str, options: &Options) -> Self {
        let mut shader = options.shaders.panel_header.clone().insert_string(
            "u_title_text".to_owned(),
            title.to_owned(),
            1,
        );
        shader.middle.parameters.r#box.size.x = self.middle.parameters.r#box.size.x;
        for child in shader.after.iter_mut() {
            child.middle.parameters.r#box.size.x = shader.middle.parameters.r#box.size.x;
        }
        self.after.push(shader);
        self
    }

    pub fn wrap_panel_footer(mut self, buttons: Vec<PanelFooterButton>, options: &Options) -> Self {
        let mut shader = options.shaders.panel_footer.clone();
        shader.middle.parameters.r#box.size.x = self.middle.parameters.r#box.size.x;
        for child in shader.after.iter_mut() {
            child.middle.parameters.r#box.size.x = shader.middle.parameters.r#box.size.x;
        }
        let mut buttons = buttons
            .into_iter()
            .map(|x| x.get_button(self.middle.entity, options))
            .collect_vec();
        if buttons.len() == 2 {
            buttons[0].middle.parameters.r#box.anchor = vec2(-0.5, 0.0);
            buttons[1].middle.parameters.r#box.anchor = vec2(0.5, 0.0);
        }
        shader.after.extend(buttons.into_iter());
        self.after.push(shader);
        self
    }

    pub fn set_panel_color(self, color: Rgba<f32>) -> Self {
        self.insert_color("u_panel_color".to_owned(), color)
    }

    pub fn set_panel_body_color(self, color: Rgba<f32>) -> Self {
        self.insert_color("u_panel_body_color".to_owned(), color)
    }

    pub fn panel(
        mut self,
        r#type: PanelType,
        color: Option<Rgba<f32>>,
        resources: &Resources,
    ) -> Panel {
        match r#type {
            PanelType::Push => {
                self.middle.parameters.r#box.center = vec2(-1.0, -1.0);
            }
            PanelType::Alert => {}
            PanelType::Hint => {
                self.middle.parameters.r#box.center = vec2(1.0, -1.0);
            }
            PanelType::Stats => {
                self.middle.parameters.r#box.center = vec2(-1.0, 1.0);
            }
        }
        if let Some(color) = color {
            self = self.set_panel_color(color);
        }
        Panel {
            need_pos: self.middle.parameters.r#box.pos,
            shader: self,
            state: PanelState::Open,
            r#type,
            t: default(),
            ts: resources.global_time,
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash)]
pub enum PanelType {
    Push,
    Alert,
    Hint,
    Stats,
}

impl PanelType {
    pub fn duration(&self) -> Time {
        match self {
            PanelType::Push => 5.0,
            _ => 0.0,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum PanelState {
    Open,
    Closed,
}
#[derive(Clone)]
pub enum PanelFooterButton {
    Close,
    Select,
    Accept,
    Reroll,
    Restart,
    Buy { cost: usize },
    Custom { name: String, handler: Handler },
    Start,
}

impl PanelFooterButton {
    pub fn get_button(&self, entity: Option<legion::Entity>, options: &Options) -> ShaderChain {
        let mut button_shader = options.shaders.panel_button.clone().insert_string(
            "u_text".to_owned(),
            self.get_text().to_owned(),
            1,
        );
        button_shader.middle.parent = entity;
        button_shader.middle.entity = Some(new_entity());
        ButtonSystem::add_button_handlers(&mut button_shader.middle);
        button_shader
            .middle
            .input_handlers
            .push(self.get_input_handler());
        if let Some(update_handler) = self.get_update_handler() {
            button_shader
                .middle
                .post_update_handlers
                .insert(0, update_handler);
        }
        if let Some(update_handler) = self.get_pre_update_handler() {
            button_shader
                .middle
                .pre_update_handlers
                .push(update_handler);
        }
        match self {
            PanelFooterButton::Buy { cost } => {
                button_shader
                    .middle
                    .insert_int_ref("u_cost".to_owned(), *cost as i32);
            }
            _ => {}
        }
        button_shader
    }

    pub fn get_text(&self) -> String {
        match self {
            PanelFooterButton::Close => "Close".to_owned(),
            PanelFooterButton::Select => "Select".to_owned(),
            PanelFooterButton::Accept => "Accept".to_owned(),
            PanelFooterButton::Reroll => "Reroll".to_owned(),
            PanelFooterButton::Restart => "Restart".to_owned(),
            PanelFooterButton::Start => "Start".to_owned(),
            PanelFooterButton::Buy { cost } => format!("-{}g", cost),
            PanelFooterButton::Custom { name, .. } => name.to_owned(),
        }
    }

    pub fn get_input_handler(&self) -> Handler {
        match self {
            PanelFooterButton::Buy { cost } => {
                fn input_handler(
                    event: HandleEvent,
                    entity: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    match event {
                        HandleEvent::Click => {
                            if PanelFooterButton::buy_check(shader, world, resources) {
                                if let Some(choice) = resources.panels_data.choice_options.clone() {
                                    let index = shader.get_int("u_index") as usize;
                                    choice.do_choose(index, world, resources);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                input_handler
            }
            PanelFooterButton::Start => {
                fn input_handler(
                    event: HandleEvent,
                    _: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    match event {
                        HandleEvent::Click => {
                            if let Some(entity) = shader.parent {
                                PanelsSystem::close_alert(entity, resources);
                                BattleSystem::play_battle(world, resources);
                            }
                        }
                        _ => {}
                    }
                }
                input_handler
            }
            PanelFooterButton::Close => {
                fn input_handler(
                    event: HandleEvent,
                    _: legion::Entity,
                    shader: &mut Shader,
                    _: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    match event {
                        HandleEvent::Click => {
                            if let Some(entity) = shader.parent {
                                PanelsSystem::close_alert(entity, resources);
                            }
                        }
                        _ => {}
                    }
                }
                input_handler
            }
            PanelFooterButton::Reroll => {
                fn input_handler(
                    event: HandleEvent,
                    _: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    match event {
                        HandleEvent::Click => {
                            if ShopSystem::is_reroll_affordable(world) {
                                if let Some(entity) = shader.parent {
                                    PanelsSystem::close_alert(entity, resources);
                                    resources.panels_data.chosen_ind = None;
                                    let choice =
                                        resources.panels_data.choice_options.take().unwrap();
                                    ShopSystem::deduct_reroll_cost(world, resources);
                                    ShopSystem::show_offers_panel(resources)
                                }
                            }
                        }
                        _ => {}
                    }
                }
                input_handler
            }
            PanelFooterButton::Restart => {
                fn input_handler(
                    event: HandleEvent,
                    _: legion::Entity,
                    _: &mut Shader,
                    world: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    match event {
                        HandleEvent::Click => {
                            Game::restart(world, resources);
                        }
                        _ => {}
                    }
                }
                input_handler
            }
            PanelFooterButton::Select => {
                fn input_handler(
                    event: HandleEvent,
                    entity: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    match event {
                        HandleEvent::Click => {
                            if let Some(entity) = shader.parent {
                                let ind = shader.parameters.uniforms.try_get_int("u_index").unwrap()
                                    as usize;
                                resources.panels_data.chosen_ind = Some(ind);
                            }
                        }
                        _ => {}
                    }
                }
                input_handler
            }
            PanelFooterButton::Accept => {
                fn input_handler(
                    event: HandleEvent,
                    _: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    match event {
                        HandleEvent::Click => {
                            if let Some(chosen) = resources.panels_data.chosen_ind {
                                if let Some(entity) = shader.parent {
                                    PanelsSystem::close_alert(entity, resources);
                                    if let Some(choice) =
                                        resources.panels_data.choice_options.take()
                                    {
                                        choice.do_choose(chosen, world, resources);
                                    }
                                    resources.panels_data.chosen_ind = None;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                input_handler
            }
            PanelFooterButton::Custom { handler, .. } => *handler,
        }
    }

    fn buy_check(shader: &Shader, world: &legion::World, resources: &Resources) -> bool {
        let cost = shader.get_int("u_cost");
        let index = shader.get_int("u_index") as usize;
        let check = if let Some(choice) = resources.panels_data.choice_options.clone() {
            match &choice {
                CardChoice::ShopOffers { units, .. } => {
                    index < units.len()
                        && TeamSystem::get_state(Faction::Team, world)
                            .get_int(&VarName::Slots, world) as usize
                            > resources.shop_data.current_team_size
                }
                _ => true,
            }
        } else {
            true
        };
        check
            && ShopSystem::get_g(world) >= cost
            && !resources.panels_data.removed_inds.contains(&index)
    }

    pub fn get_update_handler(&self) -> Option<Handler> {
        match self {
            PanelFooterButton::Buy { .. } => {
                fn update_handler(
                    _: HandleEvent,
                    _: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    shader.set_active(PanelFooterButton::buy_check(shader, world, resources));
                }
                Some(update_handler)
            }
            PanelFooterButton::Accept => {
                fn update_handler(
                    _: HandleEvent,
                    _: legion::Entity,
                    shader: &mut Shader,
                    _: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    shader.set_active(resources.panels_data.chosen_ind.is_some());
                }
                Some(update_handler)
            }
            PanelFooterButton::Reroll => {
                fn update_handler(
                    _: HandleEvent,
                    _: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    _: &mut Resources,
                ) {
                    shader.set_active(ShopSystem::is_reroll_affordable(world));
                }
                Some(update_handler)
            }
            _ => None,
        }
    }

    pub fn get_pre_update_handler(&self) -> Option<Handler> {
        match self {
            PanelFooterButton::Reroll => {
                fn update_handler(
                    _: HandleEvent,
                    _: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    let cost = ShopSystem::reroll_cost(world, resources);
                    shader.parameters.uniforms.insert_string_ref(
                        "u_text".to_owned(),
                        format!("Reroll -{}g", cost),
                        1,
                    );
                }
                Some(update_handler)
            }
            _ => None,
        }
    }
}
