#![no_main]

use libfuzzer_sys::fuzz_target;
use zvec::{DataType, DocOperator, IndexType, LogLevel, MetricType, QuantizeType};

#[derive(arbitrary::Arbitrary, Debug)]
struct FuzzInput {
    value: u32,
}

fuzz_target!(|input: FuzzInput| {
    let FuzzInput { value } = input;

    let dt: DataType = DataType::from(value);
    let back: u32 = dt.into();
    let dt2: DataType = DataType::from(back);
    assert_eq!(dt, dt2, "DataType roundtrip failed for {}", value);

    let it: IndexType = IndexType::from(value);
    let back: u32 = it.into();
    let it2: IndexType = IndexType::from(back);
    assert_eq!(it, it2, "IndexType roundtrip failed for {}", value);

    let mt: MetricType = MetricType::from(value);
    let back: u32 = mt.into();
    let mt2: MetricType = MetricType::from(back);
    assert_eq!(mt, mt2, "MetricType roundtrip failed for {}", value);

    let qt: QuantizeType = QuantizeType::from(value);
    let back: u32 = qt.into();
    let qt2: QuantizeType = QuantizeType::from(back);
    assert_eq!(qt, qt2, "QuantizeType roundtrip failed for {}", value);

    let ll: LogLevel = LogLevel::from(value);
    let back: u32 = ll.into();
    let ll2: LogLevel = LogLevel::from(back);
    assert_eq!(ll, ll2, "LogLevel roundtrip failed for {}", value);

    let op: DocOperator = DocOperator::from(value);
    let back: u32 = op.into();
    let op2: DocOperator = DocOperator::from(back);
    assert_eq!(op, op2, "DocOperator roundtrip failed for {}", value);
});
