use ngrams::{Client, Corpus, CorpusInfo, CorpusStat};

#[tokio::test]
async fn get_corpus_info() {
    let client = Client::new();
    match client.get_corpus_info(Corpus::English).await {
        Ok(info) => {
            assert_eq!(
                info,
                CorpusInfo {
                    name: "English".into(),
                    label: "eng".into(),
                    stats: [
                        CorpusStat {
                            num_ngrams: 76_862_879,
                            min_year: 1470,
                            max_year: 2019,
                            min_match_count: 1,
                            max_match_count: 1_922_716_631,
                            min_total_match_count: 40,
                            max_total_match_count: 115_513_165_249,
                        },
                        CorpusStat {
                            num_ngrams: 1_604_084_580,
                            min_year: 1470,
                            max_year: 2019,
                            min_match_count: 1,
                            max_match_count: 1_446_928_350,
                            min_total_match_count: 40,
                            max_total_match_count: 82_544_506_739,
                        },
                        CorpusStat {
                            num_ngrams: 11_777_289_629,
                            min_year: 1470,
                            max_year: 2019,
                            min_match_count: 1,
                            max_match_count: 84_854_130,
                            min_total_match_count: 40,
                            max_total_match_count: 2_907_518_961,
                        },
                        CorpusStat {
                            num_ngrams: 5_089_891_990,
                            min_year: 1470,
                            max_year: 2019,
                            min_match_count: 1,
                            max_match_count: 14_391_742,
                            min_total_match_count: 40,
                            max_total_match_count: 384_260_789,
                        },
                        CorpusStat {
                            num_ngrams: 5_020_506_742,
                            min_year: 1470,
                            max_year: 2019,
                            min_match_count: 1,
                            max_match_count: 7_167_265,
                            min_total_match_count: 40,
                            max_total_match_count: 226_361_873,
                        }
                    ],
                }
            )
        }
        Err(err) => eprintln!("{err}"),
    }
}
