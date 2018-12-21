extern crate record;
#[macro_use]
extern crate registry;

use record::Record;
use std::sync::Arc;

registry! {
    AggregatorFe:
    //array,
    count,
    //records,
}

pub trait AggregatorFe {
    fn argct(&self) -> usize;
    fn state(&self, args: &[String]) -> Box<AggregatorState>;
}

pub trait AggregatorState {
    fn add(&mut self, Record);
    fn finish(self) -> Record;
    fn box_clone(&self) -> Box<AggregatorState>;
}

pub trait AggregatorBe {
    type Args: AggregatorArgs;
    type State: AggregatorBeState<<Self::Args as AggregatorArgs>::Val>;
}

pub trait AggregatorArgs {
    type Val;

    fn argct() -> usize;
    fn parse(args: &[String]) -> Self::Val;
}

pub trait AggregatorBeState<A>: Clone + Default {
    fn add(&mut self, &A, Record);
    fn finish(self, &A) -> Record;
}

impl<B: AggregatorBe + 'static> AggregatorFe for B {
    fn argct(&self) -> usize {
        return B::Args::argct();
    }

    fn state(&self, args: &[String]) -> Box<AggregatorState> {
        return Box::new(AggregatorStateImpl::<B> {
            a: Arc::from(B::Args::parse(args)),
            s: B::State::default(),
        });
    }
}

struct AggregatorStateImpl<B: AggregatorBe> {
    a: Arc<<<B as AggregatorBe>::Args as AggregatorArgs>::Val>,
    s: B::State,
}

impl<B: AggregatorBe + 'static> AggregatorState for AggregatorStateImpl<B> {
    fn add(&mut self, r: Record) {
        self.s.add(&self.a, r);
    }

    fn finish(self) -> Record {
        return self.s.finish(&self.a);
    }

    fn box_clone(&self) -> Box<AggregatorState> {
        return Box::new(AggregatorStateImpl::<B> {
            a: self.a.clone(),
            s: self.s.clone(),
        });
    }
}



pub struct ZeroArgs();

impl AggregatorArgs for ZeroArgs {
    type Val = ();

    fn argct() -> usize {
        return 0;
    }

    fn parse(args: &[String]) -> () {
        debug_assert_eq!(0, args.len());
        return ();
    }
}