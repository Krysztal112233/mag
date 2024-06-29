use clap::ArgMatches;

use crate::service;

pub async fn handler(matches: &ArgMatches) {
    async fn rocket() -> rocket::Rocket<rocket::Build> {
        rocket::build()
            .attach(service::stage_parse())
            .attach(service::stage_query())
            .attach(service::stage_async_status())
    }

    let _ = rocket().await.launch().await;
}
