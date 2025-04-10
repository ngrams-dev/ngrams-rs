// Copyright Martin Trenkmann
// https://ngrams.dev
// License: MIT

use reqwest::StatusCode;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::ops::Deref;
use std::{error, fmt};

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

    pub async fn get_ngram(&self, corpus: Corpus, id: &str) -> Result<Option<Ngram>, Error> {
        let res = internal::get(self, corpus, id).send().await?;
        match res.status() {
            StatusCode::OK => Ok(Some(res.json().await?)),
            StatusCode::NOT_FOUND => Ok(None),
            other => Err(Error::unexpected_status_code(other.as_u16())),
        }
    }

    pub async fn get_corpus_info(&self, corpus: Corpus) -> Result<CorpusInfo, Error> {
        let res = internal::get(self, corpus, "info").send().await?;
        match res.status() {
            StatusCode::OK => Ok(res.json().await?),
            other => Err(Error::unexpected_status_code(other.as_u16())),
        }
    }

    pub async fn get_total_counts(&self, corpus: Corpus) -> Result<TotalCounts, Error> {
        let res = internal::get(self, corpus, "total_counts").send().await?;
        match res.status() {
            StatusCode::OK => Ok(res.json().await?),
            other => Err(Error::unexpected_status_code(other.as_u16())),
        }
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
            Self::English => "eng",
            Self::German => "ger",
            Self::Russian => "rus",
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

        use internal::{get, ErrorResult, SearchResult};

        match get(&self.client, self.corpus, "search")
            .query(&params)
            .send()
            .await
        {
            Ok(res) => match res.status() {
                StatusCode::OK => match res.text().await {
                    Ok(text) => {
                        self.payload = text; // NgramTokenView::text backing
                        match serde_json::from_str::<SearchResult>(&self.payload) {
                            Ok(res) => {
                                if let Some(token) = res.next_page_token {
                                    self.options.max_page_count -= 1;
                                    self.next = Some(token.into());
                                } else {
                                    self.options.max_page_count = 0;
                                    self.next = None;
                                }
                                Some(Ok(PageView {
                                    query_tokens: res.query_tokens,
                                    ngrams: res.ngrams,
                                }))
                            }
                            Err(err) => Some(Err(Error::exception(err))),
                        }
                    }
                    Err(err) => Some(Err(Error::exception(err))),
                },
                StatusCode::BAD_REQUEST => match res.json::<ErrorResult>().await {
                    Ok(res) => Some(Err(Error::bad_input(BadInputError {
                        code: res.error.code,
                        query_tokens: res.query_tokens,
                    }))),
                    Err(err) => Some(Err(Error::exception(err))),
                },
                other => Some(Err(Error::unexpected_status_code(other.as_u16()))),
            },
            Err(err) => Some(Err(Error::connection(err))),
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Ngram {
    pub id: String,
    pub abs_total_match_count: u64,
    pub rel_total_match_count: f64,
    pub tokens: Vec<NgramToken>,
    pub stats: Vec<NgramStat>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NgramStat {
    pub year: u16,
    pub abs_match_count: u64,
    pub rel_match_count: f64,
}

impl NgramStat {
    pub fn new(year: u16, abs_match_count: u64, rel_match_count: f64) -> Self {
        Self {
            year,
            abs_match_count,
            rel_match_count,
        }
    }
}

impl PartialEq for NgramStat {
    fn eq(&self, other: &Self) -> bool {
        self.year == other.year
            && self.abs_match_count == other.abs_match_count
            && (self.rel_match_count - other.rel_match_count).abs() < f64::EPSILON
    }
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    source: Option<Box<dyn error::Error>>,
}

impl Error {
    pub fn new(kind: ErrorKind, source: Option<Box<dyn error::Error>>) -> Self {
        Self { kind, source }
    }

    pub fn connection(err: reqwest::Error) -> Self {
        Self::new(ErrorKind::Connection, Some(Box::new(err)))
    }

    pub fn exception(err: impl error::Error + 'static) -> Self {
        Self::new(ErrorKind::Connection, Some(Box::new(err)))
    }

    pub fn bad_input(err: BadInputError) -> Self {
        Self::new(ErrorKind::BadInput, Some(Box::new(err)))
    }

    pub fn unexpected_status_code(code: u16) -> Self {
        Self::exception(UnexpectedStatusCode(code))
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn source(&self) -> Option<&dyn error::Error> {
        self.source.as_deref()
    }

    pub fn into_bad_input_error(self) -> BadInputError {
        *self.source.unwrap().downcast::<BadInputError>().unwrap()
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.source.as_deref()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::Connection => f.write_str("connection error"),
            ErrorKind::Exception => f.write_str("unexpected error"),
            ErrorKind::BadInput => f.write_str("bad input"),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::connection(err)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    /// HTTP connection error.
    Connection,
    /// Unexpected HTTP status code or invalid JSON.
    Exception,
    /// User query or other input was invalid.
    BadInput,
}

/// https://github.com/ngrams-dev/general/wiki/REST-API#errorresponse
#[derive(Debug)]
pub struct BadInputError {
    pub code: ErrorCode,
    pub query_tokens: Option<Vec<QueryToken>>,
}

impl fmt::Display for BadInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for BadInputError {}

#[derive(Debug)]
pub struct UnexpectedStatusCode(u16);

impl fmt::Display for UnexpectedStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unexpected http status code: {}", self.0)
    }
}

impl error::Error for UnexpectedStatusCode {}

/// Subset of error code a user query could generate.
/// See https://github.com/ngrams-dev/general/wiki/REST-API#errorcode
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ErrorCode {
    #[serde(rename = "INVALID_PARAMETER.LIMIT")]
    InvalidParameterLimit,
    #[serde(rename = "INVALID_QUERY.BAD_ALTERNATION")]
    InvalidQueryBadAlternation,
    #[serde(rename = "INVALID_QUERY.BAD_COMPLETION")]
    InvalidQueryBadCompletion,
    #[serde(rename = "INVALID_QUERY.BAD_TERM_GROUP")]
    InvalidQueryBadTermGroup,
    #[serde(rename = "INVALID_QUERY.NO_TERM")]
    InvalidQueryNoTerm,
    #[serde(rename = "INVALID_QUERY.TOO_EXPENSIVE")]
    InvalidQueryTooExpensive,
    #[serde(rename = "INVALID_QUERY.TOO_MANY_TOKENS")]
    InvalidQueryTooManyTokens,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CorpusInfo {
    pub name: String,
    pub label: String,
    pub stats: [CorpusStat; 5],
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CorpusStat {
    pub num_ngrams: u64,
    pub min_year: u16,
    pub max_year: u16,
    pub min_match_count: u32,
    pub max_match_count: u32,
    pub min_total_match_count: u64,
    pub max_total_match_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TotalCounts {
    pub min_year: u16,
    pub max_year: u16,
    pub match_counts: [TotalCountsByYear; 5],
}

#[derive(Debug)]
pub struct TotalCountsByYear([u64; TOTAL_COUNTS_BY_YEAR_LEN]);
pub const TOTAL_COUNTS_BY_YEAR_LEN: usize = 550;

impl Deref for TotalCountsByYear {
    type Target = [u64];

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl Serialize for TotalCountsByYear {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.as_slice().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TotalCountsByYear {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let deserialized = Vec::<u64>::deserialize(deserializer)?;
        if deserialized.len() != TOTAL_COUNTS_BY_YEAR_LEN {
            let expected = TOTAL_COUNTS_BY_YEAR_LEN.to_string();
            Err(serde::de::Error::invalid_length(
                deserialized.len(),
                &expected.as_str(),
            ))
        } else {
            let mut counts = TotalCountsByYear([0; TOTAL_COUNTS_BY_YEAR_LEN]);
            counts.0.copy_from_slice(&deserialized);
            Ok(counts)
        }
    }
}

/// Internal module containing implementation details.
/// Used for benchmarking. Don't use directly.
mod internal {
    use crate::{Client, Corpus, ErrorCode, NgramLiteView, QueryToken, QueryTokenView, BASE_URL};
    use reqwest::RequestBuilder;
    use serde::Deserialize;
    use std::borrow::Cow;

    pub(crate) fn get(client: &Client, corpus: Corpus, resource: &str) -> RequestBuilder {
        client
            .inner
            .get(format!("{}/{}/{}", BASE_URL, corpus.label(), resource))
            .header("user-agent", &client.user_agent)
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct SearchResult<'a> {
        #[serde(borrow)]
        pub(crate) query_tokens: Vec<QueryTokenView<'a>>,
        pub(crate) ngrams: Vec<NgramLiteView<'a>>,
        pub(crate) next_page_token: Option<Cow<'a, str>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct ErrorResult {
        pub(crate) error: Error,
        pub(crate) query_tokens: Option<Vec<QueryToken>>,
    }

    #[derive(Debug, Deserialize)]
    pub(crate) struct Error {
        pub(crate) code: ErrorCode,
        /// Currently unused.
        #[allow(dead_code)]
        pub(crate) context: Option<String>,
    }
}

#[cfg(test)]
mod tests {
    use crate::{BadInputError, Client, Corpus, ErrorCode, ErrorKind, SearchOptions};

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
            Some(Err(err)) => match err.kind() {
                ErrorKind::Connection => panic!(),
                ErrorKind::Exception => panic!(),
                ErrorKind::BadInput => {
                    let err = err.source.unwrap().downcast::<BadInputError>().unwrap();
                    assert_eq!(err.code, ErrorCode::InvalidParameterLimit);
                    assert_eq!(err.query_tokens, None);
                }
            },
            _ => panic!(),
        }
    }
}
