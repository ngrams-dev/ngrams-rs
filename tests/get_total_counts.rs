use ngrams::{Client, Corpus};

#[tokio::test]
async fn get_total_counts() {
    let client = Client::new();
    match client.get_total_counts(Corpus::English).await {
        Ok(counts) => {
            assert_eq!(counts.min_year, 1470);
            assert_eq!(counts.max_year, 2019);
            for counts_by_year in counts.match_counts {
                assert_ne!(*counts_by_year.last().unwrap(), 0);
            }
        }
        Err(err) => eprintln!("{err}"),
    }
}
