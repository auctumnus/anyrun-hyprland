use std::fs;

use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use hyprland::data::*;
use hyprland::dispatch::*;
use hyprland::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Config {
    max_entries: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { max_entries: 3 }
    }
}

struct Context {
    config: Config,
    matcher: SkimMatcherV2,
}

#[init]
fn init(config_dir: RString) -> Context {
    let config = if let Ok(content) = fs::read_to_string(format!("{}/windows.ron", config_dir)) {
        ron::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    };

    let matcher = SkimMatcherV2::default();

    Context { config, matcher }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "Windows".into(),
        icon: "video-display".into(),
    }
}

fn address_to_u64(address: &hyprland::shared::Address) -> u64 {
    // SAFETY: Hyprland always returns addresses of the form "0x...",
    // so this shouldn't crash.
    u64::from_str_radix(&address.to_string()[2..], 16).unwrap()
}

#[get_matches]
fn get_matches(input: RString, ctx: &mut Context) -> RVec<Match> {
    let mut entries = Clients::get().map_or(vec![], |clients| {
        clients
            .filter_map(|client| {
                let score = ctx.matcher.fuzzy_match(&client.title, &input).unwrap_or(0)
                    + ctx.matcher.fuzzy_match(&client.class, &input).unwrap_or(0);

                if score > 0 {
                    let id = address_to_u64(&client.address);
                    Some((client, score, id))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    });

    entries.sort_by_cached_key(|c| c.1);

    entries.truncate(ctx.config.max_entries);

    entries
        .into_iter()
        .map(|(client, _, id)| Match {
            title: client.title.into(),
            description: ROption::RNone,
            use_pango: false,
            icon: ROption::RNone,
            id: ROption::RSome(id),
        })
        .collect()
}

fn find_client(selection_id: u64) -> Option<Client> {
    let mut clients = Clients::get().ok()?;
    clients.find(|Client { address, .. }| address_to_u64(address) == selection_id)
}

fn switch_to_client(client: Client) -> std::result::Result<(), hyprland::shared::HyprError> {
    use DispatchType::FocusWindow;
    use WindowIdentifier::Address;

    Dispatch::call(FocusWindow(Address(client.address)))
}

#[handler]
fn handler(selection: Match) -> HandleResult {
    // SAFETY: We always create an id for the selection, so this cannot crash.
    let selection_id = selection.id.unwrap();
    let result = find_client(selection_id).map(switch_to_client);

    if result.is_none() {
        eprintln!("couldn't find client by address: {selection_id}");
    } else if let Some(Err(e)) = result {
        eprintln!("couldn't switch focus: {e}");
    }

    HandleResult::Close
}
