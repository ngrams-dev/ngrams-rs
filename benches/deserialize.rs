use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ngrams::{Client, Corpus, Page, PageView, SearchOptions};
use tokio::runtime::Runtime;

fn search() -> String {
    let client = Client::new();
    let options = SearchOptions::default();
    Runtime::new()
        .unwrap()
        .block_on(client.search_raw("you are * * *", Corpus::English, &options))
        .unwrap()
}

fn deserialize_page(c: &mut Criterion) {
    let json = search();
    c.bench_function("deserialize_page", |b| {
        b.iter(|| serde_json::from_str::<Page>(black_box(&json)))
    });
}

fn deserialize_page_view(c: &mut Criterion) {
    let json = search();
    c.bench_function("deserialize_page_view", |b| {
        b.iter(|| serde_json::from_str::<PageView>(black_box(&json)))
    });
}

criterion_group!(benches, deserialize_page, deserialize_page_view);
criterion_main!(benches);
