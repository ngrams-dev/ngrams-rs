// Copyright Martin Trenkmann
// https://ngrams.dev
// License: MIT

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt;

const BASE_URL: &str = "https://api.ngrams.dev";

pub struct Client {
    client: reqwest::Client,
    user_agent: String,
}

impl Client {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            user_agent: format!(
                "{}/{}/{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS
            ),
        }
    }

    pub async fn search<'a>(
        &self,
        query: &str,
        corpus: Corpus,
        options: &SearchOptions,
        storage: &'a mut StringStorage,
    ) -> Result<PageView<'a>, Error> {
        storage.0 = self
            .client
            .get(format!("{}/{}/search", BASE_URL, corpus.label()))
            .header("user-agent", &self.user_agent)
            .query(&[
                ("query", query),
                ("flags", &options.to_flags()),
                ("limit", &options.max_page_size.to_string()),
            ])
            .send()
            .await?
            .text()
            .await?;
        Ok(serde_json::from_str(&storage.0)?)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum Error {
    Http(reqwest::Error),
    Serde(serde_json::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Http(err) => err.fmt(f),
            Error::Serde(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Http(err) => err.source(),
            Error::Serde(err) => err.source(),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

pub enum Corpus {
    English,
    German,
    Russian,
}

impl Corpus {
    pub fn label(&self) -> &str {
        match self {
            Corpus::English => "eng",
            Corpus::German => "ger",
            Corpus::Russian => "rus",
        }
    }
}

pub struct SearchOptions {
    pub max_page_size: u8,
    pub case_sensitive: bool,
    pub collapse_result: bool,
    pub exclude_punctuation_marks: bool,
    pub exclude_sentence_boundary_tags: bool,
    pub dont_interpret_query_operators: bool,
    pub dont_tokenize_query_terms: bool,
    pub dont_unicode_normalize_query: bool,
}

impl SearchOptions {
    fn to_flags(&self) -> String {
        let mut flags = String::new();
        if self.case_sensitive {
            flags.push_str("cs");
        }
        if self.collapse_result {
            flags.push_str("cr");
        }
        if self.exclude_punctuation_marks {
            flags.push_str("ep");
        }
        if self.exclude_sentence_boundary_tags {
            flags.push_str("es");
        }
        if self.dont_interpret_query_operators {
            flags.push_str("ri");
        }
        if self.dont_tokenize_query_terms {
            flags.push_str("rt");
        }
        if self.dont_unicode_normalize_query {
            flags.push_str("rn");
        }
        flags
    }
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            max_page_size: 100,
            case_sensitive: false,
            collapse_result: false,
            exclude_punctuation_marks: false,
            exclude_sentence_boundary_tags: false,
            dont_interpret_query_operators: false,
            dont_tokenize_query_terms: false,
            dont_unicode_normalize_query: false,
        }
    }
}

#[derive(Default)]
pub struct StringStorage(String);

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub query_tokens: Vec<QueryToken>,
    // TODO Consider to use smallvec
    pub ngrams: Vec<NgramLite>,
    next_page_token: Option<String>,
    next_page_link: Option<String>,
}

impl Page {
    pub fn to_page_view(&self) -> PageView {
        PageView::from(self)
    }
}

impl From<&PageView<'_>> for Page {
    fn from(page: &PageView) -> Self {
        Self {
            query_tokens: page.query_tokens.iter().map(QueryToken::from).collect(),
            ngrams: page.ngrams.iter().map(NgramLite::from).collect(),
            next_page_token: page.next_page_token.map(String::from),
            next_page_link: page.next_page_link.map(String::from),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageView<'a> {
    // TODO Consider to use smallvec
    pub query_tokens: Vec<QueryTokenView<'a>>,
    // TODO Consider to use smallvec
    pub ngrams: Vec<NgramLiteView<'a>>,
    next_page_token: Option<&'a str>,
    next_page_link: Option<&'a str>,
}

impl PageView<'_> {
    pub fn to_page(&self) -> Page {
        Page::from(self)
    }
}

impl<'a> From<&'a Page> for PageView<'a> {
    fn from(page: &'a Page) -> Self {
        Self {
            query_tokens: page.query_tokens.iter().map(QueryTokenView::from).collect(),
            ngrams: page.ngrams.iter().map(NgramLiteView::from).collect(),
            next_page_token: page.next_page_token.as_deref(),
            next_page_link: page.next_page_link.as_deref(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct QueryToken {
    pub kind: QueryTokenKind,
    pub text: String,
}

impl QueryToken {
    pub fn to_query_token_view(&self) -> QueryTokenView {
        QueryTokenView::from(self)
    }
}

impl From<&QueryTokenView<'_>> for QueryToken {
    fn from(token: &QueryTokenView) -> Self {
        Self {
            kind: token.kind,
            text: token.text.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryTokenView<'a> {
    pub kind: QueryTokenKind,
    #[serde(borrow)]
    pub text: Cow<'a, str>,
}

impl QueryTokenView<'_> {
    pub fn to_query_token(&self) -> QueryToken {
        QueryToken::from(self)
    }
}

impl<'a> From<&'a QueryToken> for QueryTokenView<'a> {
    fn from(token: &'a QueryToken) -> Self {
        Self {
            kind: token.kind,
            text: Cow::Borrowed(&token.text),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QueryTokenKind {
    Term,
    Star,
    Starstar,
    StarAdj,
    StarAdp,
    StarAdv,
    StarConj,
    StarDet,
    StarNoun,
    StarNum,
    StarPron,
    StarPrt,
    StarVerb,
    SentenceStart,
    SentenceEnd,
    Slash,
    Prefix,
    TermGroup,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NgramLite {
    pub id: String,
    pub abs_total_match_count: u64,
    pub rel_total_match_count: f64,
    // TODO Consider to use smallvec
    pub tokens: Vec<NgramToken>,
    #[serde(default)]
    pub r#abstract: bool,
}

impl NgramLite {
    pub fn to_ngram_lite_view(&self) -> NgramLiteView {
        NgramLiteView::from(self)
    }
}

impl From<&NgramLiteView<'_>> for NgramLite {
    fn from(ngram: &NgramLiteView) -> Self {
        Self {
            id: ngram.id.to_string(),
            abs_total_match_count: ngram.abs_total_match_count,
            rel_total_match_count: ngram.rel_total_match_count,
            tokens: ngram.tokens.iter().map(NgramToken::from).collect(),
            r#abstract: ngram.r#abstract,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NgramLiteView<'a> {
    pub id: &'a str,
    pub abs_total_match_count: u64,
    pub rel_total_match_count: f64,
    // TODO Consider to use smallvec
    pub tokens: Vec<NgramTokenView<'a>>,
    #[serde(default)]
    pub r#abstract: bool,
}

impl NgramLiteView<'_> {
    pub fn to_ngram_lite(&self) -> NgramLite {
        NgramLite::from(self)
    }
}

impl<'a> From<&'a NgramLite> for NgramLiteView<'a> {
    fn from(ngram: &'a NgramLite) -> Self {
        Self {
            id: ngram.id.as_str(),
            abs_total_match_count: ngram.abs_total_match_count,
            rel_total_match_count: ngram.rel_total_match_count,
            tokens: ngram.tokens.iter().map(NgramTokenView::from).collect(),
            r#abstract: ngram.r#abstract,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct NgramToken {
    pub kind: NgramTokenKind,
    pub text: String,
    #[serde(default)]
    pub inserted: bool,
    #[serde(default)]
    pub completed: bool,
}

impl NgramToken {
    pub fn to_ngram_token_view(&self) -> NgramTokenView {
        NgramTokenView::from(self)
    }
}

impl From<&NgramTokenView<'_>> for NgramToken {
    fn from(token: &NgramTokenView) -> Self {
        Self {
            kind: token.kind,
            text: token.text.to_string(),
            inserted: token.inserted,
            completed: token.completed,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NgramTokenView<'a> {
    pub kind: NgramTokenKind,
    #[serde(borrow)]
    pub text: Cow<'a, str>,
    #[serde(default)]
    pub inserted: bool,
    #[serde(default)]
    pub completed: bool,
}

impl NgramTokenView<'_> {
    pub fn to_ngram_token(&self) -> NgramToken {
        NgramToken::from(self)
    }
}

impl<'a> From<&'a NgramToken> for NgramTokenView<'a> {
    fn from(token: &'a NgramToken) -> Self {
        Self {
            kind: token.kind,
            text: Cow::Borrowed(&token.text),
            inserted: token.inserted,
            completed: token.completed,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NgramTokenKind {
    Term,
    TaggedAsAdj,
    TaggedAsAdp,
    TaggedAsAdv,
    TaggedAsConj,
    TaggedAsDet,
    TaggedAsNoun,
    TaggedAsNum,
    TaggedAsPron,
    TaggedAsPrt,
    TaggedAsVerb,
    SentenceStart,
    SentenceEnd,
}

#[cfg(test)]
mod tests {
    use crate::{Client, Corpus, SearchOptions, StringStorage};

    #[tokio::test]
    async fn hello() {
        let client = Client::new();
        let options = SearchOptions::default();
        let mut buf = StringStorage::default();
        let page = client
            .search("hello *", Corpus::English, &options, &mut buf)
            .await
            .unwrap();
        assert_eq!(page.query_tokens.len(), 2);
        assert_eq!(page.ngrams.len(), 100);
    }
}
