extern crate html5ever;
extern crate regex;
extern crate reqwest;

use html5ever::tendril::*;
use html5ever::local_name;
use html5ever::tokenizer::{BufferQueue, CharacterTokens, TagToken, Token, TokenSink, TokenSinkResult, Tokenizer, StartTag, TokenizerOpts};
use regex::Regex;
use reqwest::Url;
use std::process::exit;

struct TitleFinder {
    this_is_it: bool,
}

fn clean_up_title(title: &str) -> &str {
    title.trim()
}

impl TokenSink for TitleFinder {
    type Handle = ();

    fn process_token(&mut self, token: Token, _: u64) -> TokenSinkResult<()> {
        match token {
            // Perk up for <title>
            TagToken(tag) =>
                if tag.kind == StartTag && tag.name == local_name!("title") {
                    self.this_is_it = true;
                }

            // After <title> write out the string
            CharacterTokens(txt) => {
                if self.this_is_it {
                    print!("{}\n", clean_up_title(&txt));
                    exit(0);
                }
            },

            // Ignore everything else
            _ => (),
        }

        TokenSinkResult::Continue
    }
}

fn main() {
    let msg = match std::env::args().nth(1) {
        Some(msg) => msg,
        None => exit(1),
    };

    let url = match Regex::new(r"https?://[^ ]*").unwrap().find(&msg) {
        Some(url) => url.as_str(),
        _ => exit(1),
    };

    let url = match Url::parse(&url) {
        Ok(url) => url,
        _ => exit(1),
    };

    let mut res = match reqwest::get(url) {
        Ok(res) => res,
        _ => exit(1),
    };

    let body = match res.text() {
        Ok(body) => body,
        _ => exit(1),
    };

    let tendril = ByteTendril::from_slice(body.as_bytes());

    let mut input = BufferQueue::new();
    input.push_back(tendril.try_reinterpret().unwrap());

    let mut tokenizer = Tokenizer::new(
        TitleFinder{this_is_it: false},
        TokenizerOpts{..Default::default()},
    );

    let _ = tokenizer.feed(&mut input);

    // If we get here we didn't find a <title>
    exit(1);
}
