use ecolor::Color32;

use super::*;

#[table(public, name = incubator_link, index(name = from_to, btree(columns = [from, to])))]
pub struct TIncubatorLink {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub from: u64,
    #[index(btree)]
    pub to: u64,
}

#[reducer]
fn incubator_link_add(ctx: &ReducerContext, from: u64, to: u64) -> Result<(), String> {
    ctx.player()?;
    let link_id = if let Some(link) = ctx.db.incubator_link().from_to().filter((from, to)).next() {
        link.id
    } else {
        let id = next_id(ctx);
        ctx.db
            .incubator_link()
            .insert(TIncubatorLink { id, from, to });
        id
    };
    incubator_vote_set(ctx, link_id, true)
}

#[table(public, name = incubator_vote, index(name = owner_target, btree(columns = [owner, target])))]
pub struct TIncubatorVote {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    #[index(btree)]
    pub target: u64,
    pub vote: bool,
}

#[reducer]
fn incubator_vote_set(ctx: &ReducerContext, target: u64, value: bool) -> Result<(), String> {
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

#[table(public, name = incubator_unit)]
pub struct TIncubatorUnit {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    pub name: String,
    pub description: String,
    pub hp: i32,
    pub pwr: i32,
}

#[reducer]
fn incubator_post_unit(
    ctx: &ReducerContext,
    name: String,
    description: String,
    hp: i32,
    pwr: i32,
) -> Result<(), String> {
    let player = ctx.player()?;
    ctx.db.incubator_unit().insert(TIncubatorUnit {
        id: next_id(ctx),
        owner: player.id,
        name,
        description,
        hp,
        pwr,
    });
    Ok(())
}
#[reducer]
fn incubator_delete_unit(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    if let Some(row) = ctx.db.incubator_unit().id().find(id) {
        if row.owner != player.id {
            return Err(format!("{id} not owned by {}", player.id));
        }
        ctx.db.incubator_unit().id().delete(id);
    }
    Ok(())
}

#[table(public, name = incubator_representation)]
pub struct TIncubatorRepresentation {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    #[index(btree)]
    pub data: String,
    pub description: String,
}

#[reducer]
fn incubator_post_representation(
    ctx: &ReducerContext,
    data: String,
    description: String,
) -> Result<(), String> {
    let player = ctx.player()?;
    if data.is_empty() {
        return Err("Data can't be empty".into());
    }
    if let Some(r) = ctx
        .db
        .incubator_representation()
        .data()
        .filter(&data)
        .next()
    {
        return Err(format!("Identical representation exists: id#{}", r.id));
    }
    ctx.db
        .incubator_representation()
        .insert(TIncubatorRepresentation {
            id: next_id(ctx),
            owner: player.id,
            data,
            description,
        });
    Ok(())
}
#[reducer]
fn incubator_delete_representation(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    if let Some(row) = ctx.db.incubator_representation().id().find(id) {
        if row.owner != player.id {
            return Err(format!("{id} not owned by {}", player.id));
        }
        ctx.db.incubator_representation().id().delete(id);
    }
    Ok(())
}

#[table(public, name = incubator_trigger)]
pub struct TIncubatorTrigger {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    #[index(btree)]
    pub data: String,
    pub description: String,
}

#[reducer]
fn incubator_post_trigger(
    ctx: &ReducerContext,
    data: String,
    description: String,
) -> Result<(), String> {
    let player = ctx.player()?;
    if data.is_empty() {
        return Err("Data can't be empty".into());
    }
    if let Some(r) = ctx.db.incubator_trigger().data().filter(&data).next() {
        return Err(format!("Identical trigger exists: id#{}", r.id));
    }
    ctx.db.incubator_trigger().insert(TIncubatorTrigger {
        id: next_id(ctx),
        owner: player.id,
        data,
        description,
    });
    Ok(())
}
#[reducer]
fn incubator_delete_trigger(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    if let Some(row) = ctx.db.incubator_trigger().id().find(id) {
        if row.owner != player.id {
            return Err(format!("{id} not owned by {}", player.id));
        }
        ctx.db.incubator_trigger().id().delete(id);
    }
    Ok(())
}

#[table(public, name = incubator_effect)]
pub struct TIncubatorEffect {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    #[index(btree)]
    pub data: String,
    pub description: String,
}

#[reducer]
fn incubator_post_effect(
    ctx: &ReducerContext,
    data: String,
    description: String,
) -> Result<(), String> {
    let player = ctx.player()?;
    if data.is_empty() {
        return Err("Data can't be empty".into());
    }
    if let Some(r) = ctx.db.incubator_effect().data().filter(&data).next() {
        return Err(format!("Identical effect exists: id#{}", r.id));
    }
    ctx.db.incubator_effect().insert(TIncubatorEffect {
        id: next_id(ctx),
        owner: player.id,
        data,
        description,
    });
    Ok(())
}
#[reducer]
fn incubator_delete_effect(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    if let Some(row) = ctx.db.incubator_effect().id().find(id) {
        if row.owner != player.id {
            return Err(format!("{id} not owned by {}", player.id));
        }
        ctx.db.incubator_effect().id().delete(id);
    }
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
#[reducer]
fn incubator_delete_house(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    if let Some(row) = ctx.db.incubator_house().id().find(id) {
        if row.owner != player.id {
            return Err(format!("{id} not owned by {}", player.id));
        }
        ctx.db.incubator_house().id().delete(id);
    }
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
#[reducer]
fn incubator_delete_ability(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    if let Some(row) = ctx.db.incubator_ability().id().find(id) {
        if row.owner != player.id {
            return Err(format!("{id} not owned by {}", player.id));
        }
        ctx.db.incubator_ability().id().delete(id);
    }
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
#[reducer]
fn incubator_delete_status(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    if let Some(row) = ctx.db.incubator_status().id().find(id) {
        if row.owner != player.id {
            return Err(format!("{id} not owned by {}", player.id));
        }
        ctx.db.incubator_status().id().delete(id);
    }
    Ok(())
}
