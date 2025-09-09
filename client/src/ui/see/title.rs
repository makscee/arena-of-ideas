use super::*;

/// Title trait for types that can be displayed as a colored string title.
/// This trait is limited to types that implement ToCstr and provides a default
/// implementation that just calls `.cstr()`.
pub trait SFnCstrTitle: ToCstr {
    fn cstr_title(&self, _context: &Context) -> Cstr {
        self.cstr()
    }
}

/// Integration with SeeBuilder for title functionality
impl<'a, T: SFnCstrTitle> SeeBuilder<'a, T> {
    pub fn title_button(self, ui: &mut Ui) -> Response {
        self.data().cstr_title(self.context()).button(ui)
    }

    pub fn title_label(self, ui: &mut Ui) -> Response {
        self.data().cstr_title(self.context()).label(ui)
    }

    pub fn title_cstr(self) -> Cstr {
        self.data().cstr_title(self.context())
    }
}

impl SFnCstrTitle for Action {
    fn cstr_title(&self, context: &Context) -> Cstr {
        fn add_lvl(r: &mut String, context: &Context) {
            if let Ok(lvl) = context.get_i32(VarName::lvl) {
                *r += &format!(" [tw [s lvl]][{} [b {lvl}]]", VarName::lvl.color().to_hex());
            }
        }
        match self {
            Action::use_ability => {
                let mut r = self.cstr();
                if let Ok(ability) = context.get_string(VarName::ability_name) {
                    if let Ok(color) = context.get_color(VarName::color) {
                        r += " ";
                        r += &ability.cstr_cs(color, CstrStyle::Bold);
                        add_lvl(&mut r, context);
                    }
                }
                r
            }
            _ => self.cstr(),
        }
    }
}

impl SFnCstrTitle for NArena {}
impl SFnCstrTitle for NFloorPool {}
impl SFnCstrTitle for NFloorBoss {}
impl SFnCstrTitle for NPlayer {}
impl SFnCstrTitle for NPlayerData {}
impl SFnCstrTitle for NPlayerIdentity {}
impl SFnCstrTitle for NHouse {}
impl SFnCstrTitle for NHouseColor {}
impl SFnCstrTitle for NAbilityMagic {}
impl SFnCstrTitle for NAbilityDescription {}
impl SFnCstrTitle for NAbilityEffect {}
impl SFnCstrTitle for NStatusMagic {}
impl SFnCstrTitle for NStatusDescription {}
impl SFnCstrTitle for NStatusBehavior {}
impl SFnCstrTitle for NStatusRepresentation {}
impl SFnCstrTitle for NTeam {}
impl SFnCstrTitle for NBattle {}
impl SFnCstrTitle for NMatch {}
impl SFnCstrTitle for NFusion {}
impl SFnCstrTitle for NFusionSlot {}
impl SFnCstrTitle for NUnit {}
impl SFnCstrTitle for NUnitDescription {}
impl SFnCstrTitle for NUnitStats {}
impl SFnCstrTitle for NUnitState {}
impl SFnCstrTitle for NUnitBehavior {}
impl SFnCstrTitle for NUnitRepresentation {}

impl SFnCstrTitle for Expression {}
impl SFnCstrTitle for PainterAction {}
