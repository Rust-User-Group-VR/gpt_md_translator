/* File md_chunker.rs
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


use std::{str::Split, iter::Peekable};

use tiktoken_rs::CoreBPE;

pub struct MDChunker<'a> {
    cache: String,
    tokenizer: CoreBPE,
    sys_prompt_tokens: usize,
    paragraphs: Peekable<Split<'a, &'a str>>
}

impl<'a> MDChunker<'a> {
    pub fn new(text: &'a String, tokenizer: CoreBPE, sys_prompt_tokens: usize) -> MDChunker<'a> {
        MDChunker {
            cache: String::new(),
            paragraphs: text.split("\n\n").peekable(),
            tokenizer, sys_prompt_tokens
        }
    }
}

impl<'a> Iterator for MDChunker<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next_par) = self.paragraphs.peek() {
            let cache_tokens = self.tokenizer.encode_with_special_tokens(&self.cache).len() + 4;
            let next_par_tokens = self.tokenizer.encode_with_special_tokens(next_par).len() + 4;
            if self.sys_prompt_tokens + cache_tokens + next_par_tokens >= 4097 / 2 {
                let the_next = self.cache.clone();
                self.cache = String::new();
                return Some(the_next);
            } else {
                self.cache = format!("{}{}\n\n", self.cache, self.paragraphs.next().unwrap());
            }
        }

        if self.cache.ne("") {
            let the_next = self.cache.clone();
            self.cache = String::new();
            return Some(the_next);
        } else {
            None
        }
    }
}

pub trait MDChunkable {
    fn md_chunk<'a>(&'a self, tokenizer: CoreBPE, sys_prompt_tokens: usize) -> MDChunker<'a>;
}

impl MDChunkable for String {
    fn md_chunk<'a>(&'a self, tokenizer: CoreBPE, sys_prompt_tokens: usize) -> MDChunker<'a> {
        MDChunker::new(self, tokenizer, sys_prompt_tokens)
    }
}
