use super::*;

pub trait ContentPiece {
    fn content_type(&self) -> ContentType;
    fn inner(&self) -> Vec<Box<dyn ContentPiece>>;
    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String));
    fn data(&self) -> &String;
    fn data_mut(&mut self) -> &mut String;
    fn show_node(&self, parent: u64, ui: &mut Ui, world: &mut World) {
        const FRAME: Frame = Frame {
            inner_margin: Margin::same(4.0),
            outer_margin: Margin::ZERO,
            rounding: Rounding::same(13.0),
            shadow: Shadow::NONE,
            fill: TRANSPARENT,
            stroke: STROKE_DARK,
        };
        FRAME.show(ui, |ui| {
            let ct = self.content_type();
            ui.horizontal(|ui| {
                if ct
                    .name()
                    .cstr_cs(CYAN, CstrStyle::Bold)
                    .button(ui)
                    .clicked()
                    && parent != 0
                {
                    ct.open_links(parent, world);
                }
                ct.show(self.data(), ui, world);
            });
            let id = id_by_data(self.data());
            ui.vertical(|ui| {
                for i in self.inner() {
                    i.show_node(id, ui, world);
                }
            })
        });
    }
}

fn id_by_data(data: &String) -> u64 {
    cn().db
        .content_piece()
        .data()
        .find(data)
        .map(|d| d.id)
        .unwrap_or_default()
}

impl ContentPiece for CUnit {
    fn content_type(&self) -> ContentType {
        ContentType::CUnit
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CUnit {
            data: _,
            description,
            stats,
            representation,
        } = self;
        vec![
            Box::new(description.clone()),
            Box::new(stats.clone()),
            Box::new(representation.clone()),
        ]
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }
    fn visit(&mut self, _: u64, f: fn(u64, ContentType, &mut String)) {
        let parent = id_by_data(&self.data);
        self.description.visit(parent, f);
        self.stats.visit(parent, f);
    }
}
impl ContentPiece for CUnitDescription {
    fn content_type(&self) -> ContentType {
        ContentType::CUnitDescription
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CUnitDescription { data: _, trigger } = self;
        vec![Box::new(trigger.clone())]
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }

    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
        let parent = id_by_data(&self.data);
        self.trigger.visit(parent, f);
    }
}
impl ContentPiece for CUnitStats {
    fn content_type(&self) -> ContentType {
        ContentType::CUnitStats
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        default()
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }

    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
    }
}
impl ContentPiece for CUnitRepresentation {
    fn content_type(&self) -> ContentType {
        ContentType::CUnitRepresentation
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CUnitRepresentation { data: _ } = self;
        default()
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }

    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
    }
}
impl ContentPiece for CUnitTrigger {
    fn content_type(&self) -> ContentType {
        ContentType::CUnitTrigger
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CUnitTrigger { data: _, ability } = self;
        vec![Box::new(ability.clone())]
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }

    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
        let parent = id_by_data(&self.data);
        self.ability.visit(parent, f);
    }
}
impl ContentPiece for CAbility {
    fn content_type(&self) -> ContentType {
        ContentType::CAbility
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CAbility {
            data: _,
            description,
            house,
        } = self;
        vec![Box::new(description.clone()), Box::new(house.clone())]
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }

    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
        let parent = id_by_data(&self.data);
        self.description.visit(parent, f);
        self.house.visit(parent, f);
    }
}
impl ContentPiece for CAbilityDescription {
    fn content_type(&self) -> ContentType {
        ContentType::CAbilityDescription
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CAbilityDescription {
            data: _,
            status,
            summon,
            action,
        } = self;
        vec![
            Box::new(status.clone().unwrap_or_default()),
            Box::new(summon.clone().unwrap_or_default()),
            Box::new(action.clone().unwrap_or_default()),
        ]
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }

    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
        let parent = id_by_data(&self.data);
        if let Some(c) = &mut self.status {
            c.visit(parent, f);
        }
        if let Some(c) = &mut self.summon {
            c.visit(parent, f);
        }
        if let Some(c) = &mut self.action {
            c.visit(parent, f);
        }
    }
}
impl ContentPiece for CEffect {
    fn content_type(&self) -> ContentType {
        ContentType::CEffect
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CEffect { data: _ } = self;
        default()
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }

    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
    }
}
impl ContentPiece for CStatus {
    fn content_type(&self) -> ContentType {
        ContentType::CStatus
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CStatus {
            data: _,
            description,
        } = self;
        vec![Box::new(description.clone())]
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }

    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
        let parent = id_by_data(&self.data);
        self.description.visit(parent, f);
    }
}
impl ContentPiece for CSummon {
    fn content_type(&self) -> ContentType {
        ContentType::CSummon
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CSummon { data: _, stats } = self;
        vec![Box::new(stats.clone())]
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }

    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
        let parent = id_by_data(&self.data);
        self.stats.visit(parent, f);
    }
}
impl ContentPiece for CStatusDescription {
    fn content_type(&self) -> ContentType {
        ContentType::CStatusDescription
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CStatusDescription { data: _, trigger } = self;
        vec![Box::new(trigger.clone())]
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }
    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
        let parent = id_by_data(&self.data);
        self.trigger.visit(parent, f);
    }
}
impl ContentPiece for CStatusTrigger {
    fn content_type(&self) -> ContentType {
        ContentType::CStatusTrigger
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CStatusTrigger { data: _ } = self;
        default()
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }

    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
    }
}
impl ContentPiece for CHouse {
    fn content_type(&self) -> ContentType {
        ContentType::CHouse
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CHouse { data: _ } = self;
        default()
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }

    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
    }
}
impl Default for CUnit {
    fn default() -> Self {
        Self {
            data: default(),
            description: default(),
            stats: default(),
            representation: default(),
        }
    }
}
impl Default for CUnitDescription {
    fn default() -> Self {
        Self {
            data: default(),
            trigger: default(),
        }
    }
}
impl Default for CUnitStats {
    fn default() -> Self {
        Self { data: default() }
    }
}
impl Default for CUnitRepresentation {
    fn default() -> Self {
        Self { data: default() }
    }
}
impl Default for CUnitTrigger {
    fn default() -> Self {
        Self {
            data: default(),
            ability: default(),
        }
    }
}
impl Default for CAbility {
    fn default() -> Self {
        Self {
            data: default(),
            description: default(),
            house: default(),
        }
    }
}
impl Default for CAbilityDescription {
    fn default() -> Self {
        Self {
            data: default(),
            status: Some(default()),
            summon: Some(default()),
            action: Some(default()),
        }
    }
}
impl Default for CEffect {
    fn default() -> Self {
        Self { data: default() }
    }
}
impl Default for CStatusDescription {
    fn default() -> Self {
        Self {
            data: default(),
            trigger: default(),
        }
    }
}
impl Default for CStatus {
    fn default() -> Self {
        Self {
            data: default(),
            description: default(),
        }
    }
}
impl Default for CStatusTrigger {
    fn default() -> Self {
        Self { data: default() }
    }
}
impl Default for CSummon {
    fn default() -> Self {
        Self {
            data: default(),
            stats: default(),
        }
    }
}
impl Default for CHouse {
    fn default() -> Self {
        Self { data: default() }
    }
}
