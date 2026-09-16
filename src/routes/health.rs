use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use crate::auth::AppState;

pub async fn healthz(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let read_ok = sqlx::query_scalar::<_, i64>("select count(*) from heartbeat")
        .fetch_one(&state.pool)
        .await
        .is_ok();

    let write_ok = sqlx::query(
        "insert into heartbeat (id, pinged_at) values (1, now())
         on conflict (id) do update set pinged_at = now()",
    )
    .execute(&state.pool)
    .await
    .is_ok();

    if read_ok && write_ok {
        (StatusCode::OK, "ok")
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "db error")
    }
}
