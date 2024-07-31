/* File main.rs
 *
 * Copyright (C) 2023, 2024 Riccardo Sacchetto <rsacchetto(at)nexxontech(dot)it>
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


mod config;
mod md_chunker;
mod opts;

use std::{fs::File, io::{BufReader, BufWriter, Read, Write}};

use async_openai::{
    Client, config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
        CreateChatCompletionRequestArgs, CompletionUsage, FinishReason
    }
};
use log::LevelFilter;
use simplelog::{SimpleLogger, Config};
use tiktoken_rs::cl100k_base;

use crate::{config::get_config, md_chunker::MDChunkable, opts::get_opts};

const DEFAULT_SYS_PROMPT_BASE: &str = "You shall translate the following markdown to Italian, preserving the existing formatting and avoiding any other output.";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Welcome to GPT MD Translator! ===");

    SimpleLogger::init(LevelFilter::Info, Config::default())?;

    let opts = get_opts();
    let config = get_config();
    let gpt_model = config.get_string("gptmodel")?;
    let sys_prompt_base = if let Ok(sys_prompt) = config.get_string("sysprompt") {
        log::info!("Default overridden: using `{}` as the system prompt", sys_prompt);
        sys_prompt
    } else { DEFAULT_SYS_PROMPT_BASE.into() };
    let system_prompt = if let Ok(ignore_list) = config.get_array("ignorelist") {
        if ignore_list.len() > 0 {
            let mut ignore_list_string: String = ignore_list.into_iter().map(|term| term.into_string().and_then(|term| Ok(format!("\"{}\", ", term)))).collect::<Result<_, _>>()?;
            ignore_list_string.pop();
            ignore_list_string.pop();
            log::info!("Loaded the following ignore list: {}", ignore_list_string);
            if !gpt_model.starts_with("gpt-4") {
                log::warn!("You loaded an ignore list but selected a GPT version that is lower then 4.0; unexpected behavior may occour.")
            }
            format!("{} Do not translate {}.", sys_prompt_base, ignore_list_string)
        } else {
            sys_prompt_base
        }
    } else {
        sys_prompt_base
    };

    log::trace!("Loading files...");
    let input_fn = opts.get_input_path();
    let output_fn = if opts.get_output_path().ne("") { opts.get_output_path().clone() } else {
        let mut input_fn_parts = input_fn.split_inclusive('.');
        let fn_extension = input_fn_parts.next_back().unwrap();

        format!("{}translated.{}", input_fn_parts.collect::<String>(), fn_extension)
    };
    let input_file = File::open(input_fn)?;
    let output_file = File::create(&output_fn)?;
    let mut input_reader = BufReader::new(input_file);
    let mut output_writer = BufWriter::new(output_file);
    log::info!("You are about to translate the content of `{}`.", input_fn);
    log::info!("The output will be saved to `{}`", output_fn);
    
    log::trace!("Initializing OpenAI API connection...");
    let config = OpenAIConfig::new()
        .with_api_key(config.get_string("openaitoken")?);
    let client = Client::with_config(config);

    log::trace!("Reading target text...");
    let mut text = String::new();
    input_reader.read_to_string(&mut text)?;

    log::trace!("Tokenizing target text...");
    let tokenizer = cl100k_base()?;
    let sys_prompt_tokens = tokenizer.encode_with_special_tokens(&system_prompt).len() + 4;
    let text_tokens = tokenizer.encode_with_special_tokens(&text).len() + 4;
    log::info!("Computed prompt tokens: {} + {} = {}",
        sys_prompt_tokens,
        text_tokens,
        sys_prompt_tokens + text_tokens);

    let (output_text, usage) = if sys_prompt_tokens + text_tokens > 4096 {
        log::warn!("The input file is too big ({} tokens w/prompt). It will be chunked.", sys_prompt_tokens + text_tokens);
        let mut out_buf = String::new();
        let mut usage_buf = CompletionUsage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0
        };

        for (chunk_num, chunk) in text.md_chunk(tokenizer, sys_prompt_tokens).enumerate() {
            log::trace!("Translating chunk #{}...", chunk_num);

            let transalation = translate_md(&client, &chunk, &system_prompt, &gpt_model).await?;
            out_buf = format!("{}{}", out_buf, transalation.0);
            if let Some(usage) = transalation.1 {
                usage_buf.prompt_tokens += usage.prompt_tokens;
                usage_buf.completion_tokens += usage.completion_tokens;
                usage_buf.total_tokens += usage.total_tokens;
            }
        }

        Ok((out_buf, Some(usage_buf)))
    } else {
        log::trace!("Translating text...");
        translate_md(&client, &text, &system_prompt, &gpt_model).await
    }?;

    if let Some(usage_data) = usage {
        log::info!("Actual usage (due to chunking and/or overhead):");
        log::info!("|- Prompt: {}", usage_data.prompt_tokens);
        log::info!("|- Response: {}", usage_data.completion_tokens);
        log::info!("|- Total: {}", usage_data.total_tokens);
    } else { log::warn!("No usage info received.") }

    log::trace!("Saving result...");
    let content_bytes: Vec<_> = output_text.bytes().collect();
    output_writer.write_all(&content_bytes)?;
    output_writer.flush()?;

    log::info!("Operation succeeded! Have a nice day.");
    Ok(())
}

async fn translate_md(client: &Client<OpenAIConfig>, text: &str, system_prompt: &str, model: &str) -> anyhow::Result<(String, Option<CompletionUsage>)> {
    let mut res_buf = String::new();

    let chat_msgs = vec![
        ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
            content: system_prompt.into(),
            name: None
        }),
        ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::Text(text.into()),
            name: None
        })
    ];
    let chat = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(chat_msgs)
        .build()?;

    let gpt_response = client
        .chat()
        .create(chat)
        .await?;

    for choice in gpt_response.choices.into_iter() {
        if let Some(FinishReason::Stop) = choice.finish_reason {
            if let Some(ch_content) = choice.message.content {
                res_buf = format!("{}{}\n", res_buf, ch_content);
            } else {
                return Err(anyhow::anyhow!("Received an empty response!"));
            }
        } else {
            return Err(anyhow::anyhow!("Received an unacceptable finish reason!"));
        }
    }

    Ok((res_buf, gpt_response.usage))
}
