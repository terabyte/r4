use record::Record;
use record::RecordTrait;
use registry::args::OneStringArgs;
use std::collections::HashMap;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = OneStringArgs;
    type State = HashMap<Arc<str>, i64>;

    fn names() -> Vec<&'static str> {
        return vec!["countby", "ctby", "cb"];
    }

    fn add(state: &mut HashMap<Arc<str>, i64>, a: &Arc<str>, r: Record) {
        *state.entry(r.get_path(a).expect_string()).or_insert(0) += 1;
    }

    fn finish(state: HashMap<Arc<str>, i64>, _a: &Arc<str>) -> Record {
        return Record::from_hash(state.into_iter().map(|(v, ct)| (v, Record::from(ct))).collect());
    }
}
