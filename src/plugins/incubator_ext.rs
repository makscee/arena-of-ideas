use super::*;

pub trait ContentPiece {
    fn content_type(&self) -> ContentType;
    fn fill(&mut self, parent: &dyn ContentPiece, f: fn(&dyn ContentPiece, &mut dyn ContentPiece));
    fn data(&self) -> &String;
    fn data_mut(&mut self) -> &mut String;
    fn show_children(&self, ui: &mut Ui, world: &mut World);
    fn show_node(&self, _: &dyn ContentPiece, ui: &mut Ui, world: &mut World) {
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
                {
                    // ct.open_links(parent, world);
                }
                ct.show(self.data(), ui, world);
            });
            ui.vertical(|ui| {
                self.show_children(ui, world);
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
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }
    fn fill(&mut self, _: &dyn ContentPiece, f: fn(&dyn ContentPiece, &mut dyn ContentPiece)) {
        let Self {
            data,
            description,
            stats,
            representation,
        } = self;
        let data = mem::take(data);
        let mut description = description.take().unwrap_or_default();
        let mut stats = stats.take().unwrap_or_default();
        let mut representation = representation.take().unwrap_or_default();
        description.fill(self, f);
        stats.fill(self, f);
        representation.fill(self, f);
        *self = Self {
            data,
            description: Some(description),
            stats: Some(stats),
            representation: Some(representation),
        }
    }
    fn show_children(&self, ui: &mut Ui, world: &mut World) {
        let Self {
            data: _,
            description,
            stats,
            representation,
        } = self;
        if let Some(n) = description {
            n.show_node(self, ui, world);
        }
        if let Some(n) = stats {
            n.show_node(self, ui, world);
        }
        if let Some(n) = representation {
            n.show_node(self, ui, world);
        }
    }
}
impl ContentPiece for CUnitDescription {
    fn content_type(&self) -> ContentType {
        ContentType::CUnitDescription
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }
    fn fill(&mut self, parent: &dyn ContentPiece, f: fn(&dyn ContentPiece, &mut dyn ContentPiece)) {
        f(parent, self);
        let Self { data, trigger } = self;
        let data = mem::take(data);
        let mut trigger = trigger.take().unwrap_or_default();
        trigger.fill(self, f);
        *self = Self {
            data,
            trigger: Some(trigger),
        }
    }
    fn show_children(&self, ui: &mut Ui, world: &mut World) {
        let Self { data: _, trigger } = self;
        if let Some(n) = trigger {
            n.show_node(self, ui, world);
        }
    }
}
impl ContentPiece for CUnitStats {
    fn content_type(&self) -> ContentType {
        ContentType::CUnitStats
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }
    fn fill(&mut self, parent: &dyn ContentPiece, f: fn(&dyn ContentPiece, &mut dyn ContentPiece)) {
        f(parent, self);
    }
    fn show_children(&self, _: &mut Ui, _: &mut World) {}
}
impl ContentPiece for CUnitRepresentation {
    fn content_type(&self) -> ContentType {
        ContentType::CUnitRepresentation
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }
    fn fill(&mut self, parent: &dyn ContentPiece, f: fn(&dyn ContentPiece, &mut dyn ContentPiece)) {
        f(parent, self);
    }
    fn show_children(&self, _: &mut Ui, _: &mut World) {}
}
impl ContentPiece for CUnitTrigger {
    fn content_type(&self) -> ContentType {
        ContentType::CUnitTrigger
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }
    fn fill(&mut self, parent: &dyn ContentPiece, f: fn(&dyn ContentPiece, &mut dyn ContentPiece)) {
        f(parent, self);
        let Self { data, ability } = self;
        let data = mem::take(data);
        let mut ability = ability.take().unwrap_or_default();
        ability.fill(self, f);
        *self = Self {
            data,
            ability: Some(ability),
        }
    }
    fn show_children(&self, ui: &mut Ui, world: &mut World) {
        let Self { data: _, ability } = self;
        if let Some(n) = ability {
            n.show_node(self, ui, world);
        }
    }
}
impl ContentPiece for CAbility {
    fn content_type(&self) -> ContentType {
        ContentType::CAbility
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }
    fn fill(&mut self, parent: &dyn ContentPiece, f: fn(&dyn ContentPiece, &mut dyn ContentPiece)) {
        f(parent, self);
        let Self {
            data,
            description,
            house,
        } = self;
        let data = mem::take(data);
        let mut description = description.take().unwrap_or_default();
        let mut house = house.take().unwrap_or_default();
        description.fill(self, f);
        house.fill(self, f);
        *self = Self {
            data,
            description: Some(description),
            house: Some(house),
        }
    }
    fn show_children(&self, ui: &mut Ui, world: &mut World) {
        let Self {
            data: _,
            description,
            house,
        } = self;
        if let Some(n) = description {
            n.show_node(self, ui, world);
        }
        if let Some(n) = house {
            n.show_node(self, ui, world);
        }
    }
}
impl ContentPiece for CAbilityDescription {
    fn content_type(&self) -> ContentType {
        ContentType::CAbilityDescription
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }
    fn fill(&mut self, parent: &dyn ContentPiece, f: fn(&dyn ContentPiece, &mut dyn ContentPiece)) {
        f(parent, self);
        let Self {
            data,
            status,
            summon,
            action,
        } = self;
        let data = mem::take(data);
        let mut status = status.take().unwrap_or_default();
        let mut summon = summon.take().unwrap_or_default();
        let mut action = action.take().unwrap_or_default();
        status.fill(self, f);
        summon.fill(self, f);
        action.fill(self, f);
        *self = Self {
            data,
            status: Some(status),
            summon: Some(summon),
            action: Some(action),
        }
    }
    fn show_children(&self, ui: &mut Ui, world: &mut World) {
        let Self {
            data: _,
            status,
            summon,
            action,
        } = self;
        if let Some(n) = status {
            n.show_node(self, ui, world);
        }
        if let Some(n) = summon {
            n.show_node(self, ui, world);
        }
        if let Some(n) = action {
            n.show_node(self, ui, world);
        }
    }
}
impl ContentPiece for CEffect {
    fn content_type(&self) -> ContentType {
        ContentType::CEffect
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }
    fn fill(&mut self, parent: &dyn ContentPiece, f: fn(&dyn ContentPiece, &mut dyn ContentPiece)) {
        f(parent, self);
    }
    fn show_children(&self, _: &mut Ui, _: &mut World) {}
}
impl ContentPiece for CStatus {
    fn content_type(&self) -> ContentType {
        ContentType::CStatus
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }
    fn fill(&mut self, parent: &dyn ContentPiece, f: fn(&dyn ContentPiece, &mut dyn ContentPiece)) {
        f(parent, self);
        let Self { data, description } = self;
        let data = mem::take(data);
        let mut description = description.take().unwrap_or_default();
        description.fill(self, f);
        *self = Self {
            data,
            description: Some(description),
        }
    }
    fn show_children(&self, ui: &mut Ui, world: &mut World) {
        let Self {
            data: _,
            description,
        } = self;
        if let Some(n) = description {
            n.show_node(self, ui, world);
        }
    }
}
impl ContentPiece for CSummon {
    fn content_type(&self) -> ContentType {
        ContentType::CSummon
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }
    fn fill(&mut self, parent: &dyn ContentPiece, f: fn(&dyn ContentPiece, &mut dyn ContentPiece)) {
        f(parent, self);
        let Self { data, stats } = self;
        let data = mem::take(data);
        let mut stats = stats.take().unwrap_or_default();
        stats.fill(self, f);
        *self = Self {
            data,
            stats: Some(stats),
        }
    }
    fn show_children(&self, ui: &mut Ui, world: &mut World) {
        let Self { data: _, stats } = self;
        if let Some(n) = stats {
            n.show_node(self, ui, world);
        }
    }
}
impl ContentPiece for CStatusDescription {
    fn content_type(&self) -> ContentType {
        ContentType::CStatusDescription
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }
    fn fill(&mut self, parent: &dyn ContentPiece, f: fn(&dyn ContentPiece, &mut dyn ContentPiece)) {
        f(parent, self);
        let Self { data, trigger } = self;
        let data = mem::take(data);
        let mut trigger = trigger.take().unwrap_or_default();
        trigger.fill(self, f);
        *self = Self {
            data,
            trigger: Some(trigger),
        }
    }
    fn show_children(&self, ui: &mut Ui, world: &mut World) {
        let Self { data: _, trigger } = self;
        if let Some(n) = trigger {
            n.show_node(self, ui, world);
        }
    }
}
impl ContentPiece for CStatusTrigger {
    fn content_type(&self) -> ContentType {
        ContentType::CStatusTrigger
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }
    fn fill(&mut self, parent: &dyn ContentPiece, f: fn(&dyn ContentPiece, &mut dyn ContentPiece)) {
        f(parent, self);
    }
    fn show_children(&self, _: &mut Ui, _: &mut World) {}
}
impl ContentPiece for CHouse {
    fn content_type(&self) -> ContentType {
        ContentType::CHouse
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }
    fn fill(&mut self, parent: &dyn ContentPiece, f: fn(&dyn ContentPiece, &mut dyn ContentPiece)) {
        f(parent, self);
        let Self { data, color } = self;
        let data = mem::take(data);
        let mut color = color.take().unwrap_or_default();
        color.fill(self, f);
        *self = Self {
            data,
            color: Some(color),
        }
    }
    fn show_children(&self, ui: &mut Ui, world: &mut World) {
        let Self { data: _, color } = self;
        if let Some(n) = color {
            n.show_node(self, ui, world);
        }
    }
}
impl ContentPiece for CColor {
    fn content_type(&self) -> ContentType {
        ContentType::CColor
    }
    fn data(&self) -> &String {
        &self.data
    }
    fn data_mut(&mut self) -> &mut String {
        &mut self.data
    }
    fn fill(&mut self, parent: &dyn ContentPiece, f: fn(&dyn ContentPiece, &mut dyn ContentPiece)) {
        f(parent, self);
    }
    fn show_children(&self, _: &mut Ui, _: &mut World) {}
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
        Self {
            data: default(),
            color: default(),
        }
    }
}
impl Default for CColor {
    fn default() -> Self {
        Self {
            data: MISSING_COLOR.to_hex(),
        }
    }
}
