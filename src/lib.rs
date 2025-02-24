// Copyright Martin Trenkmann
// https://ngrams.dev
// License: MIT

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt;

const BASE_URL: &str = "https://api.ngrams.dev";

#[derive(Clone)]
pub struct Client {
    inner: reqwest::Client,
    user_agent: String,
}

impl Client {
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::new(),
            user_agent: format!(
                "{}/{}/{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS
            ),
        }
    }

    pub fn search<Q: Into<String>>(
        &self,
        query: Q,
        corpus: Corpus,
        options: SearchOptions,
    ) -> Pages {
        Pages::new(self.clone(), query.into(), corpus, options)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub struct SearchOptions {
    pub max_page_size: u8,
    pub max_page_count: u32,
    pub case_sensitive: bool,
    pub collapse_result: bool,
    pub exclude_punctuation_marks: bool,
    pub exclude_sentence_boundary_tags: bool,
    pub dont_interpret_query_operators: bool,
    pub dont_tokenize_query_terms: bool,
    pub dont_unicode_normalize_query: bool,
}

impl SearchOptions {
    fn to_flags(self) -> String {
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
            max_page_count: 10,
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

pub struct Pages {
    client: Client,
    query: String,
    corpus: Corpus,
    options: SearchOptions,
    payload: String,
    next: Option<String>,
}

impl Pages {
    fn new(client: Client, query: String, corpus: Corpus, options: SearchOptions) -> Self {
        Self {
            client,
            query,
            corpus,
            options,
            payload: String::new(),
            next: None,
        }
    }

    pub async fn next(&mut self) -> Option<Result<PageView, Error>> {
        if self.options.max_page_count == 0 {
            return None;
        }

        let max_page_size = self.options.max_page_size.to_string();
        let mut params = vec![("query", self.query.as_str()), ("limit", &max_page_size)];

        let flags = self.options.to_flags();
        if !flags.is_empty() {
            params.push(("flags", &flags));
        }

        if let Some(next) = &self.next {
            params.push(("start", next));
        }

        match internal::search(&self.client, self.corpus, &params).await {
            Err(err) => Some(Err(Error::Http(err))),
            Ok(json) => {
                self.payload = json;
                match serde_json::from_str::<internal::SearchResult>(&self.payload) {
                    Err(err) => Some(Err(Error::Serde(err))),
                    Ok(result) => match result.error {
                        Some(err) => Some(Err(Error::App(AppError {
                            code: err.code,
                            query_tokens: result.query_tokens,
                        }))),
                        None => {
                            match result.next_page_token {
                                Some(token) => {
                                    self.next = Some(token.into());
                                    self.options.max_page_count -= 1;
                                }
                                None => {
                                    self.next = None;
                                    self.options.max_page_count = 0;
                                }
                            }
                            Some(Ok(PageView {
                                query_tokens: result.query_tokens.unwrap(),
                                ngrams: result.ngrams.unwrap(),
                            }))
                        }
                    },
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageView<'a> {
    #[serde(borrow)]
    pub query_tokens: Vec<QueryTokenView<'a>>,
    pub ngrams: Vec<NgramLiteView<'a>>,
}

impl PageView<'_> {
    pub fn to_page(&self) -> Page {
        Page::from(self)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NgramLiteView<'a> {
    pub id: &'a str,
    pub abs_total_match_count: u64,
    pub rel_total_match_count: f64,
    pub tokens: Vec<NgramTokenView<'a>>,
    #[serde(default)]
    pub r#abstract: bool,
}

impl NgramLiteView<'_> {
    pub fn to_ngram_lite(&self) -> NgramLite {
        NgramLite::from(self)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub query_tokens: Vec<QueryToken>,
    pub ngrams: Vec<NgramLite>,
}

impl From<&PageView<'_>> for Page {
    fn from(page: &PageView) -> Self {
        Self {
            query_tokens: page.query_tokens.iter().map(QueryToken::from).collect(),
            ngrams: page.ngrams.iter().map(NgramLite::from).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct QueryToken {
    pub kind: QueryTokenKind,
    pub text: String,
}

impl From<&QueryTokenView<'_>> for QueryToken {
    fn from(token: &QueryTokenView) -> Self {
        Self {
            kind: token.kind,
            text: token.text.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NgramLite {
    pub id: String,
    pub abs_total_match_count: u64,
    pub rel_total_match_count: f64,
    pub tokens: Vec<NgramToken>,
    #[serde(default)]
    pub r#abstract: bool,
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct NgramToken {
    pub kind: NgramTokenKind,
    pub text: String,
    #[serde(default)]
    pub inserted: bool,
    #[serde(default)]
    pub completed: bool,
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

#[derive(Debug)]
pub enum Error<'a> {
    App(AppError<'a>),
    Http(reqwest::Error),
    Serde(serde_json::Error),
}

impl Error<'_> {
    pub fn from_code(code: ErrorCode) -> Self {
        Self::App(AppError {
            code,
            query_tokens: None,
        })
    }
}

impl fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::App(err) => err.fmt(f),
            Self::Http(err) => err.fmt(f),
            Self::Serde(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error<'_> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::App(_) => None,
            Self::Http(err) => err.source(),
            Self::Serde(err) => err.source(),
        }
    }
}

/// https://github.com/ngrams-dev/general/wiki/REST-API#errorresponse
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AppError<'a> {
    pub code: ErrorCode,
    #[serde(borrow)]
    pub query_tokens: Option<Vec<QueryTokenView<'a>>>,
}

impl fmt::Display for AppError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for AppError<'_> {}

// https://github.com/ngrams-dev/general/wiki/REST-API#errorcode
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ErrorCode {
    #[serde(rename = "INVALID_PARAMETER.LIMIT")]
    InvalidParameterLimit,
    InvalidParameterStart,
    InvalidQueryBadAlternation,
    InvalidQueryBadCompletion,
    InvalidQueryBadTermGroup,
    InvalidQueryNoTerm,
    InvalidQueryTooExpensive,
    InvalidQueryTooManyTokens,
    InvalidRequestBody,
    InvalidUtf8Encoding,
    MissingParameterQuery,
}

/// Internal module containing implementation details.
/// Used for benchmarking. Don't use directly.
pub mod internal {
    use crate::{Client, Corpus, ErrorCode, NgramLiteView, QueryTokenView, BASE_URL};
    use serde::Deserialize;
    use std::borrow::Cow;

    pub async fn search(
        client: &Client,
        corpus: Corpus,
        params: &[(&str, &str)],
    ) -> Result<String, reqwest::Error> {
        client
            .inner
            .get(format!("{}/{}/search", BASE_URL, corpus.label()))
            .header("user-agent", &client.user_agent)
            .query(params)
            .send()
            .await?
            .text()
            .await
    }

    /// Union of
    /// - https://github.com/ngrams-dev/general/wiki/REST-API#errorresponse
    /// - https://github.com/ngrams-dev/general/wiki/REST-API#searchresponse
    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct SearchResult<'a> {
        pub(crate) error: Option<Error>,
        #[serde(borrow)]
        pub(crate) query_tokens: Option<Vec<QueryTokenView<'a>>>,
        pub(crate) ngrams: Option<Vec<NgramLiteView<'a>>>,
        pub(crate) next_page_token: Option<Cow<'a, str>>,
    }

    /// https://github.com/ngrams-dev/general/wiki/REST-API#error
    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Error {
        pub(crate) code: ErrorCode,
        /// Currently unused.
        pub(crate) context: Option<String>,
    }
}

#[cfg(test)]
mod tests {
    use crate::{AppError, Client, Corpus, Error, ErrorCode, SearchOptions};

    #[tokio::test]
    async fn search_and_fetch_first_three_pages() {
        let client = Client::new();

        let options = SearchOptions {
            max_page_count: 3,
            ..Default::default()
        };

        let mut pages = client.search("hello * *", Corpus::English, options);

        let mut num_ngrams = 0;
        while let Some(res) = pages.next().await {
            match res {
                Ok(page) => {
                    assert_eq!(page.query_tokens.len(), 3);
                    assert_eq!(page.ngrams.len(), 100);
                    num_ngrams += page.ngrams.len();
                }
                Err(err) => {
                    eprintln!("{err}");
                    break;
                }
            }
        }
        assert_eq!(num_ngrams, 300);
    }

    #[tokio::test]
    async fn search_and_fetch_all_pages() {
        let client = Client::new();

        let options = SearchOptions {
            max_page_count: u32::MAX,
            ..Default::default()
        };

        let mut pages = client.search("what * * day", Corpus::English, options);

        let mut num_pages = 0;
        let mut num_ngrams = 0;
        while let Some(res) = pages.next().await {
            match res {
                Ok(page) => {
                    assert_eq!(page.query_tokens.len(), 4);
                    num_ngrams += page.ngrams.len();
                    num_pages += 1;
                }
                Err(err) => {
                    eprintln!("{err}");
                    break;
                }
            }
        }
        assert_eq!(num_ngrams, 1225);
        assert_eq!(num_pages, 13);
    }

    #[tokio::test]
    async fn check_error_invalid_parameter_limit() {
        let client = Client::new();
        let options = SearchOptions {
            max_page_size: 101, // Invalid value
            ..Default::default()
        };

        let mut pages = client.search("test", Corpus::English, options);

        match pages.next().await {
            Some(Err(Error::App(AppError { code, query_tokens }))) => {
                assert_eq!(code, ErrorCode::InvalidParameterLimit);
                assert_eq!(query_tokens, None);
            }
            _ => panic!(),
        }
    }
}
