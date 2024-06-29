use rocket::get;

#[get("/<version_id>/tags")]
pub(crate) async fn version(version_id: String) {}
