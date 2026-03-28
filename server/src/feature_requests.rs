use spacetimedb::{ReducerContext, Table};

use crate::{FeatureRequest, feature_request};

#[spacetimedb::reducer]
pub fn feature_request_create(ctx: &ReducerContext, description: String) -> Result<(), String> {
    if description.is_empty() {
        return Err("Description cannot be empty".to_string());
    }
    if description.len() > 1000 {
        return Err("Description too long (max 1000 chars)".to_string());
    }

    ctx.db.feature_request().insert(FeatureRequest {
        id: 0,
        player: ctx.sender(),
        description,
        rating: 0,
        status: "proposed".to_string(),
        created_at: ctx.timestamp,
    });

    Ok(())
}

#[spacetimedb::reducer]
pub fn feature_request_accept(ctx: &ReducerContext, request_id: u64) -> Result<(), String> {
    let mut request = ctx
        .db
        .feature_request()
        .id()
        .find(request_id)
        .ok_or_else(|| format!("Feature request {} not found", request_id))?;

    request.status = "accepted".to_string();
    ctx.db.feature_request().id().update(request);
    Ok(())
}

#[spacetimedb::reducer]
pub fn feature_request_reject(
    ctx: &ReducerContext,
    request_id: u64,
    reason: String,
) -> Result<(), String> {
    let mut request = ctx
        .db
        .feature_request()
        .id()
        .find(request_id)
        .ok_or_else(|| format!("Feature request {} not found", request_id))?;

    request.status = format!("rejected: {}", reason);
    ctx.db.feature_request().id().update(request);
    Ok(())
}
