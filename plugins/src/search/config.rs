// SPDX-License-Identifier: GPL-3.0-only
// Copyright © 2023 System76

use serde::Deserialize;
use slab::Slab;
use std::collections::HashMap;

#[derive(Default, Clone, Debug)]
pub struct Config {
    matches: HashMap<String, u32>,
    definitions: Slab<Definition>,
}

impl Config {
    pub fn append(&mut self, config: RawConfig) {
        for rule in config.rules {
            let idx = self.definitions.insert(rule.action);
            for keyword in rule.matches {
                self.matches.entry(keyword).or_insert(idx as u32);
            }
        }
    }

    pub fn get(&self, word: &str) -> Option<&Definition> {
        self.matches
            .get(word)
            .and_then(|idx| self.definitions.get(*idx as usize))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RawConfig {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Rule {
    pub matches: Vec<String>,
    pub action: Definition,
}

#[derive(Debug, Deserialize, Clone)]
pub enum DisplayLine {
    // Constant label used for each result
    Label(String),

    // A Regex capture on the result (everything in parens is captured)
    // e.g. name: Capture("^.+/([^/]*)$"),
    Capture(String),

    // Same as Capture above, but with replace
    // e.g. name: Replace("^(.+)$", "http://${CAPTURE}"),
    Replace(String, String),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Definition {
    pub query: String,
    pub command: String,
    pub title: DisplayLine,
    pub detail: DisplayLine,
}

pub fn load() -> Config {
    eprintln!("load config");
    let mut config = Config::default();

    for path in pop_launcher::config::find("search") {
        let string = match std::fs::read_to_string(&path) {
            Ok(string) => string,
            Err(why) => {
                eprintln!("load config err A");
                tracing::error!("failed to read config: {}", why);
                continue;
            }
        };

        match ron::from_str::<RawConfig>(&string) {
            Ok(raw) => {
                eprintln!("raw: {:?}", raw);
                config.append(raw)
            }
            Err(why) => {
                eprintln!("load config err B: {}", why);
                tracing::error!("failed to deserialize config: {}", why);
            }
        }
    }

    eprintln!("load config: {:?}", config);

    config
}
