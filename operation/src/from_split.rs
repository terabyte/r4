use OperationBe2;
use opts::parser::OptParserView;
use opts::vals::OptionalOption;
use opts::vals::StringVecOption;
use record::Record;
use regex::Regex;
use stream::Entry;
use stream::Stream;

pub struct Impl();

#[derive(Clone)]
enum DelimiterOption {
    String(String),
    Regex(Regex),
}

declare_opts! {
    delimiter: OptionalOption<DelimiterOption>,
    keys: StringVecOption,
}

impl OperationBe2 for Impl {
    type PreOptions = PreOptions;
    type PostOptions = PostOptions;

    fn names() -> Vec<&'static str> {
        return vec!["from-split"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, PreOptions>) {
        opt.match_single(&["d", "delim"], |p, a| p.delimiter.set(DelimiterOption::String(a.clone())));
        opt.match_single(&["re", "regex"], |p, a| p.delimiter.set(DelimiterOption::Regex(Regex::new(a).unwrap())));
        opt.match_single(&["k", "keys"], |p, a| {
            for a in a.split(',') {
                p.keys.push(a.to_string());
            }
        });
    }

    fn stream(o: &PostOptions) -> Stream {
        let keys = o.keys.clone();
        let delimiter = o.delimiter.clone().unwrap_or(DelimiterOption::String(",".to_string()));

        return stream::compound(
            stream::deparse(),
            stream::closures(
                (),
                move |_s, e, w| {
                    match e {
                        Entry::Bof(_file) => {
                        }
                        Entry::Record(_r) => {
                            panic!("Unexpected record in FromSplitStream");
                        }
                        Entry::Line(line) => {
                            let mut r = Record::empty_hash();
                            let vals: Vec<&str>;
                            match &delimiter {
                                DelimiterOption::String(ref s) => {
                                    vals = line.split(s).collect();
                                }
                                DelimiterOption::Regex(ref re) => {
                                    vals = re.split(&line).collect();
                                }
                            }
                            for (k, v) in keys.iter().zip(vals) {
                                r.set_path(k, Record::from_primitive_string(v));
                            }
                            w(Entry::Record(r));
                        }
                    }
                    return true;
                },
                |_s, _w| {
                },
            ),
        );
    }
}
