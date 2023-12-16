/* File config.rs
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


use once_cell::sync::OnceCell;
use config::{Config, File, FileFormat};

use crate::opts::get_opts;

static GLOBAL_CONFIG: OnceCell<Config> = OnceCell::new();

pub fn get_config() -> &'static Config {
    match GLOBAL_CONFIG.get() {
        Some(config) => config,
        None => {
            let config_builder = Config::builder()
                .add_source(File::new(get_opts().get_config_path(), FileFormat::Toml).required(false))
		.add_source(config::Environment::with_prefix("gptmdt").prefix_separator("_").separator("_"));

            GLOBAL_CONFIG.try_insert(config_builder.build().unwrap()).unwrap()
        }
    }
}
