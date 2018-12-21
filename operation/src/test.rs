use Operation;
use StreamWrapper;
use record::FromPrimitive;
use record::Record;
use std::sync::Arc;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["test"];
}

#[derive(Default)]
pub struct Impl {
}

impl Operation for Impl {
    fn validate(&self, args: &mut Vec<String>) -> StreamWrapper {
        #[derive(Default)]
        struct Pre {
            msg: Option<String>,
        }
        let mut p = Pre::default();
        opts::parse(args, &mut p, vec![
            ("msg", 1, Box::new(|p: &mut Pre, a: &[String]| p.msg = Some(a[0].clone()))),
        ]);
        let msg = p.msg.unwrap();

        let msg: Arc<str> = Arc::from(msg);

        return StreamWrapper::new(move |os| {
            let mut n = 0;
            let msg = msg.clone();

            return os.transform_records(move |mut r| {
                n += 1;
                r.set_path("n", Record::from_primitive(n));
                r.set_path("msg", Record::from_primitive_string(msg.clone()));

                return r;
            }).parse();
        });
    }
}
