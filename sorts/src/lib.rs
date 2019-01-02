#[macro_use]
extern crate lazy_static;
extern crate record;
extern crate rand;
extern crate rand_chacha;
#[macro_use]
extern crate registry;

use record::Record;
use registry::Registrant;
use registry::args::OneStringArgs;
use registry::args::RegistryArgs;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;

pub type BoxedSort = Box<SortInbox>;

registry! {
    BoxedSort,
    lexical,
    numeric,
    shuffle,
}

pub trait SortBucket {
    fn add(&mut self, r: Record, i: usize);
    fn remove_first(&mut self) -> Option<(Record, usize)>;
    fn remove_last(&mut self) -> Option<(Record, usize)>;
    fn is_empty(&self) -> bool;
}

struct KeySortBucket<T: Clone + Ord, F: Fn(Record, usize) -> T> {
    f: F,
    next: Rc<Fn() -> Box<SortBucket>>,
    map: BTreeMap<T, Box<SortBucket>>,
}

impl<T: Clone + Ord, F: Fn(Record, usize) -> T> SortBucket for KeySortBucket<T, F> {
    fn add(&mut self, r: Record, i: usize) {
        let t = (self.f)(r.clone(), i);
        let next = &self.next;
        self.map.entry(t).or_insert_with(|| next()).add(r, i);
    }

    fn remove_first(&mut self) -> Option<(Record, usize)> {
        let first_t;
        match self.map.keys().next() {
            Some(t) => {
                first_t = t.clone();
            }
            None => {
                return None;
            }
        }

        let ret;
        let remove;
        {
            let next = self.map.get_mut(&first_t).unwrap();
            assert!(!next.is_empty());
            ret = next.remove_first();
            remove = next.is_empty();
        }

        if remove {
            self.map.remove(&first_t);
        }
        return ret;
    }

    fn remove_last(&mut self) -> Option<(Record, usize)> {
        let last_t;
        match self.map.keys().rev().next() {
            Some(t) => {
                last_t = t.clone();
            }
            None => {
                return None;
            }
        }

        let ret;
        let remove;
        {
            let next = self.map.get_mut(&last_t).unwrap();
            assert!(!next.is_empty());
            ret = next.remove_last();
            remove = next.is_empty();
        }

        if remove {
            self.map.remove(&last_t);
        }
        return ret;
    }

    fn is_empty(&self) -> bool {
        return self.map.is_empty();
    }
}

impl<T: Clone + Ord + 'static, F: Fn(Record, usize) -> T + 'static> KeySortBucket<T, F> {
    fn new(f: F, next: Rc<Fn() -> Box<SortBucket>>) -> Box<SortBucket> {
        return Box::new(KeySortBucket {
            f: f,
            next: next,
            map: BTreeMap::new(),
        });
    }
}

#[derive(Default)]
pub struct VecDequeSortBucket(VecDeque<(Record, usize)>);

impl SortBucket for VecDequeSortBucket {
    fn add(&mut self, r: Record, i: usize) {
        self.0.push_back((r, i));
    }

    fn remove_first(&mut self) -> Option<(Record, usize)> {
        return self.0.pop_front();
    }

    fn remove_last(&mut self) -> Option<(Record, usize)> {
        return self.0.pop_back();
    }

    fn is_empty(&self) -> bool {
        return self.0.is_empty();
    }
}

impl VecDequeSortBucket {
    pub fn new() -> Box<SortBucket> {
        return Box::new(VecDequeSortBucket::default());
    }
}

pub trait SortBe {
    type Args: RegistryArgs;

    fn names() -> Vec<&'static str>;
    fn new_bucket(a: &<Self::Args as RegistryArgs>::Val, next: Rc<Fn() -> Box<SortBucket>>) -> Box<SortBucket>;
}

pub trait SortInbox: Send + Sync {
    fn new_bucket(&self, next: Rc<Fn() -> Box<SortBucket>>) -> Box<SortBucket>;
    fn box_clone(&self) -> BoxedSort;
}

impl Clone for BoxedSort {
    fn clone(&self) -> BoxedSort {
        return self.box_clone();
    }
}

struct SortInboxImpl<B: SortBe> {
    a: Arc<<B::Args as RegistryArgs>::Val>,
}

impl<B: SortBe + 'static> SortInbox for SortInboxImpl<B> {
    fn new_bucket(&self, next: Rc<Fn() -> Box<SortBucket>>) -> Box<SortBucket> {
        return B::new_bucket(&self.a, next);
    }

    fn box_clone(&self) -> BoxedSort {
        return Box::new(SortInboxImpl::<B> {
            a: self.a.clone(),
        });
    }
}

pub struct SortRegistrant<B: SortBe> {
    _b: std::marker::PhantomData<B>,
}

impl<B: SortBe + 'static> Registrant<BoxedSort> for SortRegistrant<B> {
    type Args = B::Args;

    fn names() -> Vec<&'static str>{
        return B::names();
    }

    fn init2(a: <B::Args as RegistryArgs>::Val) -> BoxedSort {
        return Box::new(SortInboxImpl::<B>{
            a: Arc::new(a),
        });
    }
}

pub trait SortSimpleBe {
    type T: Clone + Ord + 'static;

    fn names() -> Vec<&'static str>;
    fn get(r: Record) -> Self::T;
}

pub struct SortBeFromSimple<B: SortSimpleBe> {
    _x: std::marker::PhantomData<B>,
}

impl<B: SortSimpleBe> SortBe for SortBeFromSimple<B> {
    type Args = OneStringArgs;

    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn new_bucket(a: &Arc<str>, next: Rc<Fn() -> Box<SortBucket>>) -> Box<SortBucket> {
        let key = a.clone();
        if key.starts_with('-') {
            return KeySortBucket::new(move |r, _i| Reverse(B::get(r.get_path(&key[1..]))), next);
        }
        return KeySortBucket::new(move |r, _i| B::get(r.get_path(&key)), next);
    }
}
