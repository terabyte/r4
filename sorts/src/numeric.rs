use record::F64SortDishonorProxy;
use record::Record;
use record::RecordTrait;
use super::SortBeFromSimple;
use super::SortRegistrant;
use super::SortSimpleBe;

pub type Impl = SortRegistrant<ImplBe>;

pub(crate) type ImplBe = SortBeFromSimple<ImplSimpleBe>;

pub struct ImplSimpleBe;

impl SortSimpleBe for ImplSimpleBe {
    type T = F64SortDishonorProxy;

    fn names() -> Vec<&'static str> {
        return vec!["n", "num", "numeric"];
    }

    fn get(r: Record) -> F64SortDishonorProxy {
        return F64SortDishonorProxy(r.coerce_f64());
    }
}
