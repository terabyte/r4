use Operation;
use StreamWrapper;
use std::sync::Arc;
use std::thread;
use stream::Entry;
use stream::Stream;
use stream::StreamTrait;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["bg"];
}

#[derive(Default)]
pub struct Impl {
}

impl Operation for Impl {
    fn validate(&self, args: &mut Vec<String>) -> StreamWrapper {
        let name = args.remove(0);
        let op = super::find(&name);
        let op = op.validate(args);
        let op = Arc::from(op);

        return StreamWrapper::new(move |os| {
            let (fe, rbe, wbe) = bgop::new(os);

            let op = op.clone();
            thread::spawn(move || {
                let os = Stream::new(wbe);
                let mut os = op.wrap(os);

                loop {
                    match rbe.read() {
                        Entry::Close() => {
                            os.write(Entry::Close());
                            return;
                        }
                        e => {
                            os.write(e);
                        }
                    }
                    if os.rclosed() {
                        rbe.rclose();
                    }
                }
            });

            return Stream::new(fe);
        });
    }
}
