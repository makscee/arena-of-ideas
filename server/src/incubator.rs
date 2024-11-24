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

#[table(public, name = content_score)]
struct TContentScore {
    #[primary_key]
    id: String,
    score: i32,
}

#[table(public, name = content_vote)]
struct TContentVote {
    #[primary_key]
    id: String,
    vote: i8,
}

#[reducer]
fn incubator_vote(ctx: &ReducerContext, id: String, vote: bool) -> Result<(), String> {
    let player_id = ctx.player()?.id;
    let vote_id = format!("{player_id}_{id}");
    let vote: i8 = if vote { 1 } else { -1 };
    let delta = if let Some(mut row) = ctx.db.content_vote().id().find(&vote_id) {
        if row.vote == vote {
            return Err("Already voted".into());
        }
        let delta = vote - row.vote;
        row.vote = vote;
        ctx.db.content_vote().id().update(row);
        delta
    } else {
        ctx.db
            .content_vote()
            .insert(TContentVote { id: vote_id, vote });
        vote
    };
    if let Some(mut link) = ctx.db.content_score().id().find(&id) {
        link.score += delta as i32;
        ctx.db.content_score().id().update(link);
    } else {
        ctx.db.content_score().insert(TContentScore {
            id,
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
    CUnit,
    CUnitDescription,
    CUnitStats,
    CUnitTrigger,
    CUnitRepresentation,
    CAbility,
    CAbilityDescription,
    CEffect,
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
    data: String,
    description: CUnitDescription,
    stats: CUnitStats,
    representation: CUnitRepresentation,
}
#[derive(SpacetimeType)]
struct CUnitDescription {
    data: String,
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
    data: String,
    description: CAbilityDescription,
    house: CHouse,
}
#[derive(SpacetimeType)]
struct CAbilityDescription {
    data: String,
    status: Option<CStatus>,
    summon: Option<CSummon>,
    action: Option<CEffect>,
}
#[derive(SpacetimeType)]
struct CHouse {
    data: String,
}
#[derive(SpacetimeType)]
struct CEffect {
    data: String,
}
#[derive(SpacetimeType)]
struct CStatus {
    data: String,
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
    data: String,
    stats: CUnitStats,
}
