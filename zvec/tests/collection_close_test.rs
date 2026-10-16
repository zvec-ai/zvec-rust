//! Tests for `Collection::close` and `Collection::try_close`.
//!
//! These tests require the zvec C library (`libzvec_c_api`) to be available.
//! Set `ZVEC_LIB_DIR` and `ZVEC_INCLUDE_DIR` environment variables before running.
//!
//! Run with: `cargo test --test collection_close_test`

use std::ptr;
use std::sync::Once;

use zvec_rust::*;

static INIT: Once = Once::new();

fn ensure_initialized() {
    INIT.call_once(|| {
        initialize(None).expect("failed to initialize zvec");
    });
}

/// Creates a fresh collection and returns it together with its data path.
fn create_test_collection(parent_dir: &std::path::Path) -> (Collection, String) {
    let dir = parent_dir.join("zvec_close_data");
    let path = dir.to_str().unwrap().to_string();
    let schema = CollectionSchema::builder("close_test")
        .add_field(FieldSchema::new("id", DataType::String, false, 0).unwrap())
        .add_field(FieldSchema::new("score", DataType::Float, true, 0).unwrap())
        .build()
        .expect("failed to build schema");

    let collection =
        Collection::create_and_open(&path, &schema, None).expect("failed to create collection");
    (collection, path)
}

fn insert_one_doc(collection: &Collection) {
    let mut doc = Doc::new().unwrap();
    doc.set_pk("pk_0");
    doc.add_string("id", "pk_0").unwrap();
    doc.add_f32("score", 0.5).unwrap();

    let docs = [&doc];
    let result = collection.insert(&docs).expect("failed to insert");
    assert_eq!(result.success_count, 1);
    collection.flush().expect("failed to flush");
}

/// Control for the reopen assertions: the engine refuses a second handle while
/// the first is live.
///
/// Without this, "reopening succeeds" would be no evidence at all — it would
/// hold just as well if the engine allowed two handles on one path and nothing
/// had been released. Observed refusal text on the pinned engine:
/// `Can't lock read-write collection: <path>/LOCK`.
#[test]
fn a_live_handle_blocks_a_second_open() {
    ensure_initialized();

    let tmp_dir = tempfile::tempdir().unwrap();
    let (collection, path) = create_test_collection(tmp_dir.path());
    insert_one_doc(&collection);

    // `Collection` is not `Debug`, so the refusal is matched rather than unwrapped.
    let blocked = match Collection::open(&path, None) {
        Ok(_) => panic!("a second handle on a live collection must be refused"),
        Err(error) => error,
    };
    assert_eq!(blocked.code, ErrorCode::InternalError);
    assert!(
        !blocked.message.is_empty(),
        "the refusal must carry the engine's reason"
    );

    // And the path is openable again only once the live handle is gone.
    collection.try_close().expect("try_close must succeed");
    Collection::open(&path, None).expect("reopen after try_close");
}

/// `try_close` releases the native handle.
///
/// The release is observable from the outside — but only because of the control
/// in [`a_live_handle_blocks_a_second_open`]: a live handle blocks a second open
/// on the same path, so a successful reopen after `try_close` means this handle
/// was released, and the documents are still there.
///
/// It does not prove that the `Drop` following `try_close` skipped its close.
/// Removing the handle swap would make that `Drop` release an already-released
/// pointer, which is undefined behaviour: the process may corrupt memory and
/// still report success here. That property stays structural — one call site,
/// the swap, and the null check in `Drop` — and observing a regression needs a
/// sanitizer build rather than an assertion in this test.
#[test]
fn try_close_releases_the_handle() {
    ensure_initialized();

    let tmp_dir = tempfile::tempdir().unwrap();
    let (collection, path) = create_test_collection(tmp_dir.path());
    insert_one_doc(&collection);

    collection.try_close().expect("try_close must succeed");

    let reopened = Collection::open(&path, None).expect("failed to reopen collection");
    assert_eq!(reopened.stats().unwrap().doc_count, 1);
}

/// `close` keeps its existing behavior: it releases the handle and reports
/// success.
#[test]
fn close_releases_the_handle() {
    ensure_initialized();

    let tmp_dir = tempfile::tempdir().unwrap();
    let (collection, path) = create_test_collection(tmp_dir.path());
    insert_one_doc(&collection);

    collection.close().expect("close must succeed");

    let reopened = Collection::open(&path, None).expect("failed to reopen collection");
    assert_eq!(reopened.stats().unwrap().doc_count, 1);
}

/// A failed native close is reported to the caller instead of being dropped.
///
/// The C API rejects a null handle with `ZVEC_ERROR_INVALID_ARGUMENT`; a null
/// handle is the one close failure that can be produced deterministically
/// against the released library.
#[test]
fn try_close_reports_the_native_error_code() {
    ensure_initialized();

    let collection = unsafe { Collection::from_raw(ptr::null_mut()) };
    let error = collection
        .try_close()
        .expect_err("a null handle must not be reported as success");

    assert_eq!(error.code, ErrorCode::InvalidArgument);
    // `check_error` falls back to "unknown error" when the C library stored no
    // message, so the fallback is exactly what a broken message path produces.
    assert!(
        !error.message.is_empty() && error.message != "unknown error",
        "the caller must receive the message the C library stored: {error:?}"
    );
}
