use OperationBe2;
use aggregator::AggregatorState;
use opts::parser::OptParserView;
use opts::vals::UnvalidatedOption;
use record::Record;
use stream::Entry;
use stream::Stream;

pub struct Impl();

declare_opts! {
    aggs: UnvalidatedOption<Vec<(String, Box<AggregatorState>)>>,
}

impl OperationBe2 for Impl {
    type PreOptions = PreOptions;
    type PostOptions = PostOptions;

    fn names() -> Vec<&'static str> {
        return vec!["aggregate"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, PreOptions>) {
        aggregator::REGISTRY.labelled_single_options(&mut opt.sub(|p| &mut p.aggs), &["a", "agg", "aggregator"]);
    }

    fn stream(o: &PostOptions) -> Stream {
        return stream::compound(
            stream::parse(),
            stream::closures(
                o.aggs.clone(),
                |s, e, _w| {
                    match e {
                        Entry::Bof(_file) => {
                        }
                        Entry::Record(r) => {
                            for (_, ref mut state) in s.iter_mut() {
                                state.add(r.clone());
                            }
                        }
                        Entry::Line(_line) => {
                            panic!("Unexpected line in AggregateStream");
                        }
                    }
                    return true;
                },
                |s, w| {
                    let mut r = Record::empty_hash();
                    for (label, state) in s.into_iter() {
                        r.set_path(&label, state.finish());
                    }
                    w(Entry::Record(r));
                },
            ),
        );
    }
}
