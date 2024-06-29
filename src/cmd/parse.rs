use crate::{
    archive::flatten_decompress,
    fetcher::{modrinth::ModrinthFetcher, Fetcher},
    tag::{data::MinecraftTagData, merge, walk},
};
use clap::ArgMatches;
use indicatif::{MultiProgress, ProgressBar};
use indicatif_log_bridge::LogWrapper;
use std::fs;

pub async fn handler(matches: &ArgMatches) {
    let logger = env_logger::Builder::from_env(env_logger::Env::default()).build();

    let arg_version_id = matches.get_one::<String>("modrinth").unwrap();
    let arg_output = matches.get_one::<String>("output");
    let arg_format = matches.get_one::<String>("format").unwrap();

    let tmp = ModrinthFetcher::try_new()
        .unwrap()
        .download(arg_version_id)
        .await
        .unwrap();

    let result = flatten_decompress(tmp.path()).unwrap();

    let mut tag_result = Vec::new();

    let multi = MultiProgress::new();

    LogWrapper::new(multi.clone(), logger).try_init().unwrap();
    let bar = multi.add(ProgressBar::new(result.len().try_into().unwrap()));

    for ele in result {
        tag_result.push(walk(ele.path()).await.collect::<Vec<_>>());
        bar.inc(1);
    }
    bar.finish_and_clear();

    let tag_result = merge(tag_result);

    let result = match arg_format.to_owned() {
        s if s == "markdown" => to_markdown(tag_result, matches).await,
        s if s == "json" => to_json(tag_result, matches).await,
        _ => unreachable!(),
    };

    match arg_output {
        Some(path) => fs::write(path, result).unwrap(),
        None => println!("{}", result),
    };

    tmp.cleanup().unwrap();
}

async fn to_markdown(mut tags: Vec<MinecraftTagData>, matches: &ArgMatches) -> String {
    tags.sort_by(|a, b| {
        b.namespace.cmp(&a.namespace).cmp(
            &b.tag_name
                .cmp(&a.tag_name)
                .cmp(&b.tag_type.cmp(&a.tag_type)),
        )
    });

    let ids = matches
        .get_many::<String>("modrinth")
        .unwrap()
        .cloned()
        .collect::<Vec<_>>();

    let file_list = {
        let mut list = Vec::new();
        for ele in ids.iter() {
            let filename = ModrinthFetcher::try_new()
                .unwrap()
                .fetch(ele)
                .await
                .unwrap()
                .filename;

            list.push(format!("- {}", filename))
        }
        list.join("\n")
    };

    let ids = ids.join(", ");

    let mut output = Vec::new();
    output.push(format!("# Tag tabel for version id: {}", ids));
    output.push("This markdown file contains follow files:".to_string());
    output.push(file_list);

    let table = {
        let mut table = Vec::new();
        table.push("|Namespace|Tag Type|Tag Name|Tag Value|".to_string());
        table.push("|:--:|:--:|:--:|:--:|".to_string());
        for ele in tags {
            let value = ele
                .tag_data
                .flatten_value()
                .into_iter()
                .map(|it| "`".to_string() + &it + "`")
                .collect::<Vec<_>>()
                .join(", ");

            let str = format!(
                "|{}|{}|{}|{}|",
                ele.namespace, ele.tag_type, ele.tag_name, value
            );
            table.push(str);
        }
        table.join("\n")
    };
    output.push(table);
    output.join("\n\n")
}

async fn to_json(tags: Vec<MinecraftTagData>, matches: &ArgMatches) -> String {
    serde_json::to_string_pretty(&tags).unwrap()
}
