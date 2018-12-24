use OperationBe;
use StreamWrapper;
use SubOperationOption;
use opts::BooleanOption;
use opts::OptParserView;
use opts::OptionTrait;
use std::sync::Arc;
use stream::Stream;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["chain"];
}

#[derive(Default)]
pub struct Impl {
}

declare_opts! {
    keep_bof: BooleanOption,
    cmds: CmdsOption,
}

impl OperationBe for Impl {
    type PreOptions = PreOptions;
    type PostOptions = PostOptions;

    fn options<'a>(mut opt: OptParserView<'a, PreOptions>) {
        opt.sub(|p| &mut p.cmds).match_extra_hard(CmdsOption::push);
        opt.sub(|p| &mut p.keep_bof).match_zero(&["keep-bof"], BooleanOption::set);
        opt.sub(|p| &mut p.keep_bof).match_zero(&["no-keep-bof"], BooleanOption::clear);
    }

    fn get_extra(o: &PostOptions) -> &Vec<String> {
        return &o.cmds.extra;
    }

    fn stream(o: &PostOptions) -> Stream {
        let keep_bof = o.keep_bof;
        return o.cmds.wrs.iter().rev().fold(Stream::id(), |mut s, wr| {
            if !keep_bof {
                s = Stream::compound(Stream::drop_bof(), s);
            }
            return Stream::compound(wr.stream(), s)
        });
    }
}

#[derive(Default)]
struct CmdsOption(Vec<String>);

impl OptionTrait for CmdsOption {
    type ValidatesTo = CmdsOptions;

    fn validate(self) -> CmdsOptions {
        let mut iter = self.0.into_iter();
        let mut cmds = Vec::new();
        'TOP: loop {
            let mut cmd = Vec::new();
            loop {
                match iter.next() {
                    None => {
                        cmds.push(cmd);
                        break 'TOP;
                    }
                    Some(first) => {
                        if first == "|" {
                            cmds.push(cmd);
                            continue 'TOP;
                        }
                        cmd.push(first);
                    }
                }
            }
        }

        let mut extra = None;
        let mut wrs = Vec::new();
        for cmd in cmds {
            let so = SubOperationOption::of(cmd).validate();
            match extra {
                None => {
                    extra = Some(so.extra);
                }
                Some(_) => {
                    if so.extra.len() > 0 {
                        panic!();
                    }
                }
            }
            wrs.push(so.wr);
        }

        return CmdsOptions {
            extra: extra.unwrap(),
            wrs: wrs,
        };
    }
}

impl CmdsOption {
    fn push(&mut self, a: &[String]) {
        self.0.extend_from_slice(a);
    }
}

#[derive(Clone)]
struct CmdsOptions {
    extra: Vec<String>,
    wrs: Vec<Arc<StreamWrapper>>,
}