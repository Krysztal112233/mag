use rocket::{fairing::AdHoc, routes};

mod async_status;
mod parse;
mod query;

pub fn stage_query() -> AdHoc {
    AdHoc::on_ignite("`Query` endpoint", |r| async move {
        r.mount("/query", routes![query::version])
    })
}

pub fn stage_parse() -> AdHoc {
    AdHoc::on_ignite("`Parse` endpoint", |r| async move { r })
}

pub fn stage_async_status() -> AdHoc {
    AdHoc::on_ignite("`Async Status` endpoint", |r| async move { r })
}
