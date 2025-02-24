use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ngrams::{internal, Client, Corpus, Page, PageView};
use tokio::runtime::Runtime;

fn search() -> String {
    let client = Client::new();
    let params = &[("query", "you are * * *")];
    Runtime::new()
        .unwrap()
        .block_on(internal::search(&client, Corpus::English, params))
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
