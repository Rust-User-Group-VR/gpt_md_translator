/* File opts.rs
 *
 * Copyright (C) 2023 Riccardo Sacchetto <rsacchetto(at)nexxontech(dot)it>
 *
 * This file is part of GPT MD Translator.
 *
 * GPT MD Translator is free software: you can redistribute it and/or modify it under the terms of
 * the GNU General Public License as published by the Free Software Foundation,
 * either version 3 of the License, or (at your option) any later version.
 *
 * GPT MD Translator is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
 * more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * GPT MD Translator. If not, see <https://www.gnu.org/licenses/>. 
 */


use bpaf::{short, construct, Parser};
use once_cell::sync::OnceCell;

#[derive(Clone, Debug)]
pub struct Opts {
    config_path: String,
    input_path: String,
    output_path: String,
}

impl Opts {
    pub fn get_config_path(&self) -> &String {
        &self.config_path
    }

    pub fn get_input_path(&self) -> &String {
        &self.input_path
    }

    pub fn get_output_path(&self) -> &String {
        &self.output_path
    }
}

static GLOBAL_OPTS: OnceCell<Opts> = OnceCell::new();

fn read_opts() -> Opts {
    let config_path = short('c')
        .long("config")
        .help("Path to the config file (optional: defaults to \"./Settings.toml\")")
        .argument("CONFIG")
        .fallback(String::from("Settings.toml"));

    let input_path = short('i')
        .long("input")
        .help("Path to the file with the content to translate")
        .argument("INPUT")
        .fallback(String::from("./input.md"));

    let output_path = short('o')
        .long("output")
        .help("Path to the destination file (will be created if it doesn't exist; optional: defaults to \"[inputfile].translated.md\")")
        .argument("OUTPUT")
        .fallback(String::new());

    construct!(Opts { config_path, input_path , output_path})
        .to_options()
        .version(env!("CARGO_PKG_VERSION"))
        .descr("gpt_md_translator - Use AI to translate your text while preserving the Markdown formatting")
        .footer("Copyright 2023 (C) Riccardo Sacchetto - Licenza GNU GPLv3.0")
        .run()
}

pub fn get_opts() -> &'static Opts {
    match GLOBAL_OPTS.get() {
        Some(opts) => opts,
        None => {
            GLOBAL_OPTS.try_insert(read_opts()).unwrap()
        }
    }
}
