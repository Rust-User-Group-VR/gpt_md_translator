# GPT MD Translator

A simple tool that helps you translate any Markdown file by leveraging the power of the GPT LLM.

This program was originally developed to aid in the process of translation of "The Rust Programming Language", but is made available to anyone under the terms of the GPLv3 license.

## How to use

GPT MD Translator can easily be compiled targeting the muslc library by running `nix build` in the root of the project (N.B.: First make sure that the Nix package manager is available and configured to execute the `nix` commands and to support the Flake interface).

### Command line reference

GPT MD Translator is a CLI tool and, as such, exposes a couple of parameters that should be known to the user: to get a list of them along with a short description, just run `gpt_md_translator --help` and you'll get the following output:

```text
Usage: gpt_md_translator [-c=CONFIG] [-i=INPUT] [-o=OUTPUT]

Available options:
    -c, --config=CONFIG  Path to the config file (optional: defaults to "./Settings.toml")
    -i, --input=INPUT    Path to the file with the content to translate
    -o, --output=OUTPUT  Path to the destination file (will be created if it doesn't exist; optional: defaults to "[inputfile].translated.md")
    -h, --help           Prints help information
    -V, --version        Prints version information
```

### Config file

GPT MD Translator needs a configuration file that can be automatically retrieved from the working directory under the name of `Settings.toml` or can be manually specified by running the executable with the `-c` (`--config`) parameter.

The format of this file is defined as follows:

```toml
# Your OpenAI API token
openaitoken = "..."

# The model of the GPT LLM you want to target
# (The latest GPT 3.5 as of writing is "gpt-3.5-turbo-1106")
gptmodel = "gpt-4"

# The system prompt that will be passed to GPT right before the Markdown file;
# if you leave it empty or you omit it, a sane default one will be used
sysprompt = "Can you translate the following markdown to Italian, preserving the existing formatting and avoiding any other output?"

# A list of terms that GPT should ignore while generating the translation.
# It will be passed right after the system prompt, but is not guaranteed to work (especially with GPT < 4)
# If you leave it empty or omit it, GPT will only receive the base prompt
ignorelist = [ "term1", "term2" ]
```

As an alternative, every one of these option can be passed as an environment variable with the `GPTMDT` prefix.

## How to hack, extend and contribute

Along with the scripts needed to build a working executable, the Flake present in this repository is equipped with everything that is necessary to spawn a full-fledged environment useful to hack on the code; therefore, you can get the correct versions of `cargo` and `rustc` just by typing `nix develop` in your shell.

Once everything is set up, you can execute a debug binary by launching `cargo run -- [parameters]`.

In case you want to contribute your changes, you then only have to commit your improvements, create a Pull Request and wait for it to be merged.
