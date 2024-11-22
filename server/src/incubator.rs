use strum_macros::Display;

use super::*;

#[table(public, name = content_piece)]
struct TContentPiece {
    #[primary_key]
    id: u64,
    owner: u64,
    #[unique]
    data: String,
    #[index(btree)]
    t: SContentType,
}

#[table(public, name = content_link)]
struct TContentLink {
    #[primary_key]
    from_to: String,
    score: i32,
}

#[table(public, name = content_vote)]
struct TContentVotes {
    #[primary_key]
    id: String,
    #[index(btree)]
    owner: u64,
    vote: i8,
}

#[reducer]
fn incubator_link_vote(ctx: &ReducerContext, from_to: String, vote: bool) -> Result<(), String> {
    let player_id = ctx.player()?.id;
    let vote: i8 = if vote { 1 } else { -1 };
    let delta = if let Some(mut row) = ctx.db.content_vote().id().find(&from_to) {
        if row.vote == vote {
            return Err("Already voted".into());
        }
        let delta = vote - row.vote;
        row.vote = vote;
        ctx.db.content_vote().id().update(row);
        delta
    } else {
        ctx.db.content_vote().insert(TContentVotes {
            id: from_to.clone(),
            owner: player_id,
            vote,
        });
        vote
    };
    if let Some(mut link) = ctx.db.content_link().from_to().find(&from_to) {
        link.score += delta as i32;
        ctx.db.content_link().from_to().update(link);
    } else {
        ctx.db.content_link().insert(TContentLink {
            from_to,
            score: vote as i32,
        });
    }
    Ok(())
}

#[reducer]
fn incubator_post(ctx: &ReducerContext, t: SContentType, data: String) -> Result<(), String> {
    let player_id = ctx.player()?.id;
    if let Some(piece) = ctx.db.content_piece().data().find(&data) {
        return Err(format!(
            "Identical content piece exists: {t} id#{}",
            piece.id
        ));
    }
    ctx.db.content_piece().insert(TContentPiece {
        id: next_id(ctx),
        data,
        t,
        owner: player_id,
    });
    Ok(())
}

#[reducer]
fn incubator_delete(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let player_id = ctx.player()?.id;
    if let Some(piece) = ctx.db.content_piece().id().find(id) {
        if piece.owner != player_id {
            return Err(format!("Piece#{id} not owned by {player_id}"));
        }
        ctx.db.content_piece().id().delete(id);
    } else {
        return Err(format!("Piece#{id} not found"));
    }
    Ok(())
}

#[derive(SpacetimeType, Display)]
pub enum SContentType {
    Data,
    CUnit,
    CUnitDescription,
    CUnitStats,
    CUnitTrigger,
    CUnitRepresentation,
    CAbility,
    CAbilityDescription,
    CAbilityEffect,
    CHouse,
    CStatus,
    CStatusDescription,
    CStatusTrigger,
    CSummon,
}

#[table(public, name = units)]
struct TUnits {
    unit: CUnit,
}
#[derive(SpacetimeType)]
struct CUnit {
    name: String,
    description: CUnitDescription,
    stats: CUnitStats,
    representation: CUnitRepresentation,
}
#[derive(SpacetimeType)]
struct CUnitDescription {
    text: String,
    trigger: CUnitTrigger,
}
#[derive(SpacetimeType)]
struct CUnitStats {
    data: String,
}
#[derive(SpacetimeType)]
struct CUnitTrigger {
    data: String,
    ability: CAbility,
}
#[derive(SpacetimeType)]
struct CUnitRepresentation {
    data: String,
}
#[derive(SpacetimeType)]
struct CAbility {
    name: String,
    description: CAbilityDescription,
    house: CHouse,
}
#[derive(SpacetimeType)]
struct CAbilityDescription {
    text: String,
    effect: CAbilityEffect,
}
#[derive(SpacetimeType)]
struct CHouse {
    data: String,
}
#[derive(SpacetimeType)]
enum CAbilityEffect {
    Status(CStatus),
    Summon(CSummon),
    Action(String),
}
#[derive(SpacetimeType)]
struct CStatus {
    name: String,
    description: CStatusDescription,
}
#[derive(SpacetimeType)]
struct CStatusDescription {
    data: String,
    trigger: CStatusTrigger,
}
#[derive(SpacetimeType)]
struct CStatusTrigger {
    data: String,
}
#[derive(SpacetimeType)]
struct CSummon {
    name: String,
    stats: CUnitStats,
}
