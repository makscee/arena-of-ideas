use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Condition {
    Always,
    Not {
        condition: Box<Condition>,
    },
    UnitHasStatus {
        who: Who,
        #[serde(rename = "status")]
        status_type: StatusName,
    },
    UnitInjured {
        who: Who,
    },
    InRange {
        max_distance: Coord,
    },
    Chance {
        percent: Expr,
    },
    Equal {
        a: Box<Expr>,
        b: Box<Expr>,
    },
    Less {
        a: Box<Expr>,
        b: Box<Expr>,
    },
    More {
        a: Box<Expr>,
        b: Box<Expr>,
    },
    Clan {
        clan: Clan,
        count: usize,
    },
    HasVar {
        name: VarName,
    },
    Faction {
        who: Who,
        faction: Faction,
    },
    And {
        a: Box<Condition>,
        b: Box<Condition>,
    }
}

impl Default for Condition {
    fn default() -> Self {
        Self::Always
    }
}
