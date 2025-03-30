use ngrams::{Client, Corpus, Ngram, NgramStat, NgramToken, NgramTokenKind};

#[tokio::test]
async fn get_ngram() {
    let client = Client::new();
    let ngram = client
        .get_ngram(Corpus::English, "f2036997e2ba2ab5ba39ecc6c8d5a19f")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        ngram,
        Ngram {
            id: "f2036997e2ba2ab5ba39ecc6c8d5a19f".into(),
            abs_total_match_count: 15751,
            rel_total_match_count: 7.502425888716453e-9,
            tokens: vec![
                NgramToken {
                    kind: NgramTokenKind::Term,
                    text: "hello".to_string(),
                    inserted: false,
                    completed: false,
                },
                NgramToken {
                    kind: NgramTokenKind::Term,
                    text: "world".to_string(),
                    inserted: false,
                    completed: false,
                }
            ],
            stats: vec![
                NgramStat::new(1880, 21, 4.889791590572001e-9),
                NgramStat::new(1900, 17, 2.1000682472890535e-9),
                NgramStat::new(1901, 1, 1.1991136888826875e-10),
                NgramStat::new(1928, 1, 1.5290386430477573e-10),
                NgramStat::new(1929, 1, 1.5233041677389224e-10),
                NgramStat::new(1938, 1, 1.4780102539695689e-10),
                NgramStat::new(1949, 1, 1.1013769794641906e-10),
                NgramStat::new(1953, 1, 1.1258132161048289e-10),
                NgramStat::new(1955, 1, 1.0437787704048938e-10),
                NgramStat::new(1961, 3, 2.377666015519791e-10),
                NgramStat::new(1962, 4, 3.0094466043637716e-10),
                NgramStat::new(1963, 2, 1.410087554215183e-10),
                NgramStat::new(1964, 2, 1.4988829935346764e-10),
                NgramStat::new(1965, 1, 6.794147604413925e-11),
                NgramStat::new(1966, 6, 3.9587249907112429e-10),
                NgramStat::new(1967, 7, 4.1950271233125335e-10),
                NgramStat::new(1968, 17, 9.982357419312094e-10),
                NgramStat::new(1969, 1, 5.859767324583967e-11),
                NgramStat::new(1970, 6, 3.53550894600002e-10),
                NgramStat::new(1971, 3, 1.7706214861390619e-10),
                NgramStat::new(1972, 8, 4.785977602510226e-10),
                NgramStat::new(1973, 1, 5.842118544651718e-11),
                NgramStat::new(1974, 3, 1.7895077931960267e-10),
                NgramStat::new(1975, 1, 5.7952381955346269e-11),
                NgramStat::new(1976, 2, 1.1000398799492752e-10),
                NgramStat::new(1977, 3, 1.6283318668704998e-10),
                NgramStat::new(1978, 8, 4.318189576962546e-10),
                NgramStat::new(1979, 3, 1.6052781784516737e-10),
                NgramStat::new(1980, 9, 4.810732578122874e-10),
                NgramStat::new(1981, 8, 4.34659019805142e-10),
                NgramStat::new(1982, 5, 2.7056537902971098e-10),
                NgramStat::new(1983, 6, 3.130932620770014e-10),
                NgramStat::new(1984, 21, 1.0668596253877303e-9),
                NgramStat::new(1985, 44, 2.1710015051005258e-9),
                NgramStat::new(1986, 41, 2.0203680087397567e-9),
                NgramStat::new(1987, 36, 1.7619503587725143e-9),
                NgramStat::new(1988, 80, 3.8542110448806908e-9),
                NgramStat::new(1989, 64, 2.9434911893139596e-9),
                NgramStat::new(1990, 135, 6.012408194792335e-9),
                NgramStat::new(1991, 105, 4.846783570549088e-9),
                NgramStat::new(1992, 221, 9.792217864397809e-9),
                NgramStat::new(1993, 130, 5.765143143474609e-9),
                NgramStat::new(1994, 182, 7.967024920900617e-9),
                NgramStat::new(1995, 172, 7.291703170225742e-9),
                NgramStat::new(1996, 200, 8.583733301245634e-9),
                NgramStat::new(1997, 195, 8.452194766144627e-9),
                NgramStat::new(1998, 175, 7.453005273180486e-9),
                NgramStat::new(1999, 215, 9.033388569417678e-9),
                NgramStat::new(2000, 316, 1.2114700266745328e-8),
                NgramStat::new(2001, 243, 9.356507878327009e-9),
                NgramStat::new(2002, 429, 1.550350345516692e-8),
                NgramStat::new(2003, 502, 1.7375838185590433e-8),
                NgramStat::new(2004, 339, 1.14623280699441e-8),
                NgramStat::new(2005, 587, 2.101654653223783e-8),
                NgramStat::new(2006, 443, 1.5241384297042939e-8),
                NgramStat::new(2007, 499, 1.6795417030071738e-8),
                NgramStat::new(2008, 660, 2.1868222403056879e-8),
                NgramStat::new(2009, 644, 2.2091717619467e-8),
                NgramStat::new(2010, 653, 2.3353041213841275e-8),
                NgramStat::new(2011, 879, 3.095878049256371e-8),
                NgramStat::new(2012, 743, 2.0527844621343127e-8),
                NgramStat::new(2013, 882, 2.40630330285879e-8),
                NgramStat::new(2014, 779, 2.4044848043877065e-8),
                NgramStat::new(2015, 784, 2.7851956500220638e-8),
                NgramStat::new(2016, 1021, 3.6383337945993438e-8),
                NgramStat::new(2017, 1109, 3.83446189706865e-8),
                NgramStat::new(2018, 1231, 4.4514539588252368e-8),
                NgramStat::new(2019, 838, 3.489779960898464e-8),
            ],
        }
    )
}
