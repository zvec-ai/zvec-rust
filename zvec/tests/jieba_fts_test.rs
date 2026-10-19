//! End-to-end test for the `jieba` FTS tokenizer.
//!
//! zvec embeds cppjieba's default dictionary and HMM model (LZ4-compressed)
//! in `libzvec_c_api`, so Chinese full-text search needs no dict directory
//! to ship, discover, or register: `initialize(None)` plus
//! `IndexParams::fts(Some("jieba"), ..)` is enough. A custom dictionary can
//! still be selected per field through the `jieba_dict_dir` extra param, or
//! process-wide through the `ZVEC_JIEBA_DICT_DIR` environment variable.
//!
//! Requires the zvec C library (`libzvec_c_api`). Set `ZVEC_LIB_DIR` before
//! running. Run with: `cargo test --test jieba_fts_test`

use std::sync::Once;
use zvec_rust::*;

static INIT: Once = Once::new();

fn ensure_initialized() {
    INIT.call_once(|| {
        initialize(None).expect("failed to initialize zvec");
    });
}

#[test]
fn jieba_tokenizer_search_chinese_text() {
    ensure_initialized();

    let tmp_dir = tempfile::tempdir().unwrap();
    let dir = tmp_dir.path().join("zvec_jieba");

    // Schema with an FTS-indexed string field using the jieba tokenizer.
    let schema = CollectionSchema::builder("jieba_fts_test")
        .add_field(FieldSchema::new("id", DataType::String, false, 0).unwrap())
        .add_indexed_field(
            "content",
            DataType::String,
            IndexParams::fts(Some("jieba"), None, None).unwrap(),
        )
        .build()
        .expect("failed to build schema");

    let collection = Collection::create_and_open(dir.to_str().unwrap(), &schema, None)
        .expect("failed to create collection");

    let texts = [
        "向量数据库支持全文检索",
        "结巴分词是常用的中文分词工具",
        "全文搜索通常基于倒排索引",
    ];
    let mut docs = Vec::new();
    for (i, text) in texts.iter().enumerate() {
        let mut doc = Doc::new().unwrap();
        doc.set_pk(&format!("pk_{}", i));
        doc.add_string("id", &format!("pk_{}", i)).unwrap();
        doc.add_string("content", text).unwrap();
        docs.push(doc);
    }
    let doc_refs: Vec<&Doc> = docs.iter().collect();
    let result = collection.insert(&doc_refs).unwrap();
    assert_eq!(result.success_count, texts.len() as u64);
    collection.flush().expect("flush before query");

    // "数据库" is a single jieba token inside "向量数据库".
    let mut fts = Fts::new().unwrap();
    fts.set_match_string("数据库").unwrap();
    let query = SearchQuery::fts("content", &fts, 10).unwrap();

    let results = collection.query(&query).expect("jieba FTS query must run");
    let got: Vec<String> = results
        .iter()
        .map(|d| d.get_string("content").unwrap().unwrap())
        .collect();
    assert_eq!(got, vec!["向量数据库支持全文检索"]);

    // A term jieba segments out of the second document only.
    let mut fts = Fts::new().unwrap();
    fts.set_match_string("中文分词").unwrap();
    let query = SearchQuery::fts("content", &fts, 10).unwrap();

    let results = collection.query(&query).expect("jieba FTS query must run");
    let got: Vec<String> = results
        .iter()
        .map(|d| d.get_string("content").unwrap().unwrap())
        .collect();
    assert_eq!(got, vec!["结巴分词是常用的中文分词工具"]);
}
