#[macro_use]
extern crate lazy_static;
extern crate record;
#[macro_use]
extern crate registry;

use record::Record;
use registry::Registrant;
use registry::args::RegistryArgs;
use std::sync::Arc;

pub type BoxedDeaggregator = Box<DeaggregatorInbox>;

registry! {
    BoxedDeaggregator,
    split,
    unarray,
    unhash,
}

trait DeaggregatorBe {
    type Args: RegistryArgs;

    fn names() -> Vec<&'static str>;
    fn deaggregate(a: &<Self::Args as RegistryArgs>::Val, r: Record) -> Vec<Vec<(Arc<str>, Record)>>;
}

pub trait DeaggregatorInbox: Send + Sync {
    fn deaggregate(&self, r: Record) -> Vec<Vec<(Arc<str>, Record)>>;
    fn box_clone(&self) -> BoxedDeaggregator;
}

impl Clone for BoxedDeaggregator {
    fn clone(&self) -> BoxedDeaggregator {
        return self.box_clone();
    }
}

struct DeaggregatorInboxImpl<B: DeaggregatorBe> {
    a: Arc<<B::Args as RegistryArgs>::Val>,
}

impl<B: DeaggregatorBe + 'static> DeaggregatorInbox for DeaggregatorInboxImpl<B> {
    fn deaggregate(&self, r: Record) -> Vec<Vec<(Arc<str>, Record)>> {
        return B::deaggregate(&self.a, r);
    }

    fn box_clone(&self) -> BoxedDeaggregator {
        return Box::new(DeaggregatorInboxImpl::<B> {
            a: self.a.clone(),
        });
    }
}

struct DeaggregatorRegistrant<B: DeaggregatorBe> {
    _b: std::marker::PhantomData<B>,
}

impl<B: DeaggregatorBe + 'static> Registrant<BoxedDeaggregator> for DeaggregatorRegistrant<B> {
    type Args = B::Args;

    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn init2(a: <B::Args as RegistryArgs>::Val) -> BoxedDeaggregator {
        return Box::new(DeaggregatorInboxImpl::<B>{
            a: Arc::new(a),
        });
    }
}
