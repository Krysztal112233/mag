use clap::{arg, builder::PossibleValue, ArgAction, ArgGroup, Command};
use human_panic::setup_panic;

mod archive;
mod cmd;
mod fetcher;
mod service;
mod tag;

#[rocket::main]
async fn main() {
    let matches = app().get_matches();

    if let Some(matches) = matches.subcommand_matches("query") {
        setup_panic!();
        cmd::match_query(matches);
    } else if let Some(matches) = matches.subcommand_matches("server") {
        env_logger::init();
        cmd::service::handler(matches).await;
    } else if let Some(matches) = matches.subcommand_matches("parse") {
        setup_panic!();
        cmd::parse::handler(matches).await;
    }
}

fn app() -> Command {
    Command::new("mag")
        .arg_required_else_help(true)
        .version(env!("CARGO_PKG_VERSION"))
        .author("KrysztalHuang <krysztal.huang@outlook.com>")
        .about("M(inecraft) (T)ag Utils.")
        .subcommand(Command::new("query").about("Query tag from database"))
        .subcommand(
            Command::new("server")
                .about("Start mag server service")
                .args([arg!(-c --config "MagServer config")]),
        )
        .subcommand(
            Command::new("parse")
                .about("Parse and export data from a Minecraft mod")
                .args([
                    arg!(-m --modrinth      "Modrinth version id").action(ArgAction::Set),
                    arg!(-o --output        "Output file name").action(ArgAction::Set),
                    arg!(-f --format        "Output format")
                        .value_parser([
                            PossibleValue::new("markdown").help("Output as markdown format"),
                            PossibleValue::new("json").help("Output as json format"),
                            // PossibleValue::new("sql").help("Output as sql format"),
                        ])
                        .action(ArgAction::Set)
                        .default_value("markdown")
                        .hide_possible_values(false),
                ])
                .group(
                    ArgGroup::new("input_method")
                        .args(["modrinth"])
                        .required(true),
                ),
        )
}
