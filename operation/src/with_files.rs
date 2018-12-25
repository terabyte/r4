use OperationBe;
use StreamWrapper;
use SubOperationOption;
use opts::parser::OptParserView;
use opts::vals::OptionalStringOption;
use record::Record;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;

pub struct Impl();

declare_opts! {
    fk: OptionalStringOption,
    op: SubOperationOption,
}

impl OperationBe for Impl {
    type PreOptions = PreOptions;
    type PostOptions = PostOptions;

    fn names() -> Vec<&'static str> {
        return vec!["with-files"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, PreOptions>) {
        opt.sub(|p| &mut p.fk).match_single(&["fk", "file-key"], OptionalStringOption::set);
        opt.sub(|p| &mut p.op).match_extra_hard(SubOperationOption::push);
    }

    fn get_extra(o: &PostOptions) -> &Vec<String> {
        return &o.op.extra;
    }

    fn stream(o: &PostOptions) -> Stream {
        struct StreamState {
            fk: Arc<str>,
            sub_wr: Arc<StreamWrapper>,
            substream: Option<Stream>,
        };
        impl StreamState {
            fn close(&mut self, w: &mut FnMut(Entry) -> bool) {
                if let Some(substream) = self.substream.take() {
                    substream.close(w);
                }
            }

            fn open(&mut self, file: Option<Arc<str>>) -> &mut Stream {
                let fk = self.fk.clone();
                let fv = match file {
                    Some(file) => Record::from_arcstr(file),
                    None => Record::null(),
                };
                let sub_wr = self.sub_wr.clone();
                return self.substream.get_or_insert_with(move || {
                    return stream::compound(
                        sub_wr.stream(),
                        stream::transform_records(move |mut r| {
                            r.set_path(&fk, fv.clone());
                            return r;
                        }),
                    );
                });
            }
        }

        return stream::closures(
            StreamState {
                fk: o.fk.as_ref().map(|s| Arc::from(s as &str)).unwrap_or(Arc::from("FILE")),
                sub_wr: o.op.wr.clone(),
                substream: None,
            },
            |s, e, w| {
                match e {
                    Entry::Bof(ref file) => {
                        s.close(w);
                        s.open(Some(file.clone()));
                        return w(Entry::Bof(file.clone()));
                    },
                    Entry::Record(r) => {
                        s.open(None).write(Entry::Record(r), w);
                    }
                    Entry::Line(line) => {
                        s.open(None).write(Entry::Line(line), w);
                    }
                }
                // Again, sad, but we don't really know what will happen in
                // future substreams even if one substream is rclosed.
                return true;
            },
            |mut s, w| {
                s.close(w);
            },
        );
    }
}