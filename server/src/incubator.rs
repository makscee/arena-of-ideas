use ecolor::Color32;

use super::*;

#[derive(SpacetimeType, Clone, Copy)]
pub enum SIncubatorType {
    UnitName,
    UnitStats,
    UnitRepresentation,
    UnitTrigger,
    House,
    Ability,
    AbilityEffect,
    Status,
    StatusTrigger,
}

impl SIncubatorType {
    fn check_owner(self, ctx: &ReducerContext, owner: u64, id: u64) -> Result<(), String> {
        let entry_owner = match self {
            SIncubatorType::UnitName => {
                ctx.db
                    .incubator_unit_name()
                    .id()
                    .find(id)
                    .context_str("Entry not found")?
                    .owner
            }
            SIncubatorType::UnitStats => {
                ctx.db
                    .incubator_unit_stats()
                    .id()
                    .find(id)
                    .context_str("Entry not found")?
                    .owner
            }
            SIncubatorType::UnitRepresentation => {
                ctx.db
                    .incubator_unit_representation()
                    .id()
                    .find(id)
                    .context_str("Entry not found")?
                    .owner
            }
            SIncubatorType::UnitTrigger => {
                ctx.db
                    .incubator_unit_trigger()
                    .id()
                    .find(id)
                    .context_str("Entry not found")?
                    .owner
            }
            SIncubatorType::House => {
                ctx.db
                    .incubator_house()
                    .id()
                    .find(id)
                    .context_str("Entry not found")?
                    .owner
            }
            SIncubatorType::Ability => {
                ctx.db
                    .incubator_ability()
                    .id()
                    .find(id)
                    .context_str("Entry not found")?
                    .owner
            }
            SIncubatorType::AbilityEffect => {
                ctx.db
                    .incubator_ability_effect()
                    .id()
                    .find(id)
                    .context_str("Entry not found")?
                    .owner
            }
            SIncubatorType::Status => {
                ctx.db
                    .incubator_status()
                    .id()
                    .find(id)
                    .context_str("Entry not found")?
                    .owner
            }
            SIncubatorType::StatusTrigger => {
                ctx.db
                    .incubator_status_trigger()
                    .id()
                    .find(id)
                    .context_str("Entry not found")?
                    .owner
            }
        };
        if entry_owner != owner {
            Err(format!("Entry#{id} not owned by {owner}"))
        } else {
            Ok(())
        }
    }
}

#[reducer]
fn incubator_delete(ctx: &ReducerContext, id: u64, t: SIncubatorType) -> Result<(), String> {
    let player = ctx.player()?;
    t.check_owner(ctx, player.id, id)?;
    match t {
        SIncubatorType::UnitName => ctx.db.incubator_unit_name().id().delete(id),
        SIncubatorType::UnitStats => ctx.db.incubator_unit_stats().id().delete(id),
        SIncubatorType::UnitRepresentation => {
            ctx.db.incubator_unit_representation().id().delete(id)
        }
        SIncubatorType::UnitTrigger => ctx.db.incubator_unit_trigger().id().delete(id),
        SIncubatorType::House => ctx.db.incubator_house().id().delete(id),
        SIncubatorType::Ability => ctx.db.incubator_ability().id().delete(id),
        SIncubatorType::AbilityEffect => ctx.db.incubator_ability_effect().id().delete(id),
        SIncubatorType::Status => ctx.db.incubator_status().id().delete(id),
        SIncubatorType::StatusTrigger => ctx.db.incubator_status_trigger().id().delete(id),
    };
    TIncubatorLink::clear_id(ctx, &id.to_string());
    Ok(())
}

#[table(public, name = incubator_link, index(name = from_to, btree(columns = [from, to])))]
pub struct TIncubatorLink {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub from: String,
    #[index(btree)]
    pub from_type: SIncubatorType,
    #[index(btree)]
    pub to: String,
    #[index(btree)]
    pub to_type: SIncubatorType,
    pub score: i32,
}
impl TIncubatorLink {
    fn clear_id(ctx: &ReducerContext, id: &str) {
        ctx.db.incubator_link().from().delete(id);
        ctx.db.incubator_link().to().delete(id);
    }
}

#[reducer]
fn incubator_link_add(
    ctx: &ReducerContext,
    from: String,
    from_type: SIncubatorType,
    to: String,
    to_type: SIncubatorType,
) -> Result<(), String> {
    ctx.player()?;
    let link_id = if let Some(link) = ctx
        .db
        .incubator_link()
        .from_to()
        .filter((&from, &to))
        .next()
    {
        link.id
    } else {
        let id = next_id(ctx);
        ctx.db.incubator_link().insert(TIncubatorLink {
            id,
            from: from.clone(),
            to: to.clone(),
            score: 0,
            from_type,
            to_type,
        });
        id
    };
    incubator_vote_set(ctx, link_id, 1)
}

#[table(public, name = incubator_vote, index(name = owner_target, btree(columns = [owner, target])))]
pub struct TIncubatorVote {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    #[index(btree)]
    pub target: u64,
    pub vote: i32,
}

#[reducer]
fn incubator_vote_set(ctx: &ReducerContext, target: u64, value: i32) -> Result<(), String> {
    let player = ctx.player()?;
    if let Some(mut vote) = ctx
        .db
        .incubator_vote()
        .owner_target()
        .filter((player.id, target))
        .next()
    {
        if vote.vote == value {
            return Err("Already voted".into());
        }
        vote.vote = value;
        ctx.db.incubator_vote().id().update(vote);
    } else {
        ctx.db.incubator_vote().insert(TIncubatorVote {
            id: next_id(ctx),
            owner: player.id,
            target,
            vote: value,
        });
    }

    Ok(())
}

#[table(public, name = incubator_unit_name)]
pub struct TIncubatorUnitName {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    pub name: String,
}

#[reducer]
fn incubator_post_unit_name(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let player = ctx.player()?;
    ctx.db.incubator_unit_name().insert(TIncubatorUnitName {
        id: next_id(ctx),
        owner: player.id,
        name,
    });
    Ok(())
}

#[table(public, name = incubator_unit_stats)]
pub struct TIncubatorUnitStats {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    pub pwr: i32,
    pub hp: i32,
}

#[reducer]
fn incubator_post_unit_stats(ctx: &ReducerContext, pwr: i32, hp: i32) -> Result<(), String> {
    let player = ctx.player()?;
    ctx.db.incubator_unit_stats().insert(TIncubatorUnitStats {
        id: next_id(ctx),
        owner: player.id,
        pwr,
        hp,
    });
    Ok(())
}

#[table(public, name = incubator_unit_representation)]
pub struct TIncubatorUnitRepresentation {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    #[index(btree)]
    pub data: String,
}

#[reducer]
fn incubator_post_unit_representation(ctx: &ReducerContext, data: String) -> Result<(), String> {
    let player = ctx.player()?;
    if data.is_empty() {
        return Err("Data can't be empty".into());
    }
    if let Some(r) = ctx
        .db
        .incubator_unit_representation()
        .data()
        .filter(&data)
        .next()
    {
        return Err(format!("Identical representation exists: id#{}", r.id));
    }
    ctx.db
        .incubator_unit_representation()
        .insert(TIncubatorUnitRepresentation {
            id: next_id(ctx),
            owner: player.id,
            data,
        });
    Ok(())
}

#[table(public, name = incubator_unit_trigger)]
pub struct TIncubatorUnitTrigger {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    #[index(btree)]
    pub data: String,
}

#[reducer]
fn incubator_post_unit_trigger(ctx: &ReducerContext, data: String) -> Result<(), String> {
    let player = ctx.player()?;
    if data.is_empty() {
        return Err("Data can't be empty".into());
    }
    if let Some(r) = ctx.db.incubator_unit_trigger().data().filter(&data).next() {
        return Err(format!("Identical trigger exists: id#{}", r.id));
    }
    ctx.db
        .incubator_unit_trigger()
        .insert(TIncubatorUnitTrigger {
            id: next_id(ctx),
            owner: player.id,
            data,
        });
    Ok(())
}

#[table(public, name = incubator_house, index(name = name_color, btree(columns = [name, color])))]
pub struct TIncubatorHouse {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    pub name: String,
    pub color: String,
}

#[reducer]
fn incubator_post_house(ctx: &ReducerContext, name: String, color: String) -> Result<(), String> {
    let player = ctx.player()?;
    if let Some(e) = Color32::from_hex(&color).err() {
        return Err(format!("Failed to parse color: {e:?}"));
    }
    if let Some(r) = ctx
        .db
        .incubator_house()
        .name_color()
        .filter((&name, &color))
        .next()
    {
        return Err(format!("Identical house exists: id#{}", r.id));
    }
    ctx.db.incubator_house().insert(TIncubatorHouse {
        id: next_id(ctx),
        owner: player.id,
        name,
        color,
    });
    Ok(())
}

#[table(public, name = incubator_ability, index(name = name_description, btree(columns = [name, description])))]
pub struct TIncubatorAbility {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    pub name: String,
    pub description: String,
}

#[reducer]
fn incubator_post_ability(
    ctx: &ReducerContext,
    name: String,
    description: String,
) -> Result<(), String> {
    let player = ctx.player()?;
    if let Some(r) = ctx
        .db
        .incubator_ability()
        .name_description()
        .filter((&name, &description))
        .next()
    {
        return Err(format!("Identical ability exists: id#{}", r.id));
    }
    ctx.db.incubator_ability().insert(TIncubatorAbility {
        id: next_id(ctx),
        owner: player.id,
        name,
        description,
    });
    Ok(())
}

#[table(public, name = incubator_ability_effect)]
pub struct TIncubatorAbilityEffect {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    #[index(btree)]
    pub data: String,
}

#[reducer]
fn incubator_post_ability_effect(ctx: &ReducerContext, data: String) -> Result<(), String> {
    let player = ctx.player()?;
    if let Some(r) = ctx
        .db
        .incubator_ability_effect()
        .data()
        .filter(&data)
        .next()
    {
        return Err(format!("Identical ability effect exists: id#{}", r.id));
    }
    ctx.db
        .incubator_ability_effect()
        .insert(TIncubatorAbilityEffect {
            id: next_id(ctx),
            owner: player.id,
            data,
        });
    Ok(())
}

#[table(public, name = incubator_status, index(name = name_description_polarity, btree(columns = [name, description, polarity])))]
pub struct TIncubatorStatus {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    pub name: String,
    pub description: String,
    pub polarity: i8,
}

#[reducer]
fn incubator_post_status(
    ctx: &ReducerContext,
    name: String,
    description: String,
    polarity: i8,
) -> Result<(), String> {
    let player = ctx.player()?;
    if let Some(r) = ctx
        .db
        .incubator_status()
        .name_description_polarity()
        .filter((&name, &description, polarity))
        .next()
    {
        return Err(format!("Identical status exists: id#{}", r.id));
    }
    ctx.db.incubator_status().insert(TIncubatorStatus {
        id: next_id(ctx),
        owner: player.id,
        name,
        description,
        polarity,
    });
    Ok(())
}

#[table(public, name = incubator_status_trigger)]
pub struct TIncubatorStatusTrigger {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    #[index(btree)]
    pub data: String,
}

#[reducer]
fn incubator_post_status_trigger(ctx: &ReducerContext, data: String) -> Result<(), String> {
    let player = ctx.player()?;
    if let Some(r) = ctx
        .db
        .incubator_status_trigger()
        .data()
        .filter(&data)
        .next()
    {
        return Err(format!("Identical status trigger exists: id#{}", r.id));
    }
    ctx.db
        .incubator_status_trigger()
        .insert(TIncubatorStatusTrigger {
            id: next_id(ctx),
            owner: player.id,
            data,
        });
    Ok(())
}
