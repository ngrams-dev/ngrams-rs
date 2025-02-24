use ngrams::{Client, Corpus, SearchOptions};

#[tokio::test]
async fn hello() {
    let client = Client::new();

    let options = SearchOptions {
        max_page_size: 100,
        max_page_count: 3,
        ..Default::default()
    };

    let mut pages = client.search("hello * *", Corpus::English, options);

    while let Some(res) = pages.next().await {
        match res {
            Ok(page) => {
                assert_eq!(page.query_tokens.len(), 3);
                assert_eq!(page.ngrams.len(), 100);
            }
            Err(err) => {
                eprintln!("{err}");
                break;
            }
        }
    }
}
