extern crate stream;
extern crate wns;

use std::collections::VecDeque;
use std::sync::Arc;
use stream::Entry;
use stream::StreamTrait;
use wns::WaitNotifyState;

#[derive(Clone)]
pub enum BofOrWrite {
    Bof(Arc<str>),
    Write(Entry),
}

struct OneBuffer {
    buf: VecDeque<Option<BofOrWrite>>,
    rclosed: bool,
}

impl OneBuffer {
    fn new() -> Self {
        return OneBuffer {
            buf: VecDeque::new(),
            rclosed: false,
        };
    }
}

struct BgopState {
    os_closed: bool,
    fe_to_be: OneBuffer,
    be_to_fe: OneBuffer,
}

impl BgopState {
    fn new() -> Self {
        return BgopState {
            os_closed: false,
            fe_to_be: OneBuffer::new(),
            be_to_fe: OneBuffer::new(),
        };
    }
}

#[derive(Clone)]
pub struct BgopBe {
    state: Arc<WaitNotifyState<BgopState>>,
}

impl BgopBe {
    pub fn read(&self) -> Option<BofOrWrite> {
        return self.state.await(&mut |buffers| {
            if let Some(maybe) = buffers.fe_to_be.buf.pop_front() {
                return (Some(maybe), true);
            }
            return (None, false);
        });
    }

    pub fn rclose(&self) {
        self.state.write(|buffers| {
            buffers.fe_to_be.rclosed = true;
            buffers.fe_to_be.buf.clear();
        });
    }

    fn enqueue(&mut self, e: BofOrWrite) {
        return self.state.await(&mut |buffers| {
            if buffers.be_to_fe.rclosed {
                return (Some(()), false);
            }
            if buffers.be_to_fe.buf.len() < 1024 {
                buffers.be_to_fe.buf.push_back(Some(e.clone()));
                return (Some(()), true);
            }
            return (None, false);
        });
    }
}

impl StreamTrait for BgopBe {
    fn bof(&mut self, file: &str) {
        self.enqueue(BofOrWrite::Bof(Arc::from(file)));
    }

    fn write(&mut self, e: Entry) {
        self.enqueue(BofOrWrite::Write(e));
    }

    fn rclosed(&mut self) -> bool {
        return self.state.read(|buffers| {
            return buffers.be_to_fe.rclosed;
        });
    }

    fn close(&mut self) {
        self.state.write(|buffers| {
            buffers.be_to_fe.buf.push_back(None);
        });
    }
}

pub struct BgopFe {
    os: Box<FnMut(Option<BofOrWrite>) -> bool>,
    state: Arc<WaitNotifyState<BgopState>>,
}

impl BgopFe {
    pub fn new<OS: FnMut(Option<BofOrWrite>) -> bool + 'static>(os: OS) -> Self {
        return Self::new_box(Box::new(os));
    }

    pub fn new_box(os: Box<FnMut(Option<BofOrWrite>) -> bool>) -> Self {
        return BgopFe {
            os: os,
            state: Arc::new(WaitNotifyState::new(BgopState::new())),
        }
    }

    pub fn be(&self) -> BgopBe {
        return BgopBe {
            state: self.state.clone(),
        };
    }

    fn ferry<R, F: FnMut(&mut BgopState) -> Option<R>>(&mut self, f: &mut F) -> R {
        enum Ret<R> {
            Ferry(Vec<Option<BofOrWrite>>),
            Return(R),
        }
        loop {
            let ret = self.state.await(&mut |buffers| {
                if buffers.be_to_fe.buf.len() > 0 {
                    let mut es = Vec::new();
                    while let Some(e) = buffers.be_to_fe.buf.pop_front() {
                        if e.is_none() {
                            buffers.os_closed = true;
                        }
                        es.push(e);
                    }
                    return (Some(Ret::Ferry(es)), true);
                }

                if let Some(ret) = f(buffers) {
                    return (Some(Ret::Return(ret)), true);
                }

                return (None, false);
            });
            match ret {
                Ret::Ferry(es) => {
                    for e in es {
                        if !(self.os)(e) {
                            self.state.write(|buffers| {
                                buffers.be_to_fe.rclosed = true;
                                buffers.be_to_fe.buf.clear();
                            });
                            break;
                        }
                    }
                }
                Ret::Return(ret) => {
                    return ret;
                }
            }
        }
    }

    fn enqueue(&mut self, e: BofOrWrite) {
        return self.ferry(&mut |buffers| {
            if buffers.fe_to_be.rclosed {
                return Some(());
            }

            if buffers.fe_to_be.buf.len() < 1024 {
                buffers.fe_to_be.buf.push_back(Some(e.clone()));
                return Some(());
            }

            return None;
        });
    }
}

impl StreamTrait for BgopFe {
    fn bof(&mut self, file: &str) {
        self.enqueue(BofOrWrite::Bof(Arc::from(file)));
    }

    fn write(&mut self, e: Entry) {
        self.enqueue(BofOrWrite::Write(e));
    }

    fn rclosed(&mut self) -> bool {
        return self.state.read(|buffers| {
            return buffers.fe_to_be.rclosed;
        });
    }

    fn close(&mut self) {
        self.state.write(|buffers| {
            buffers.fe_to_be.buf.push_back(None);
        });
        self.ferry(&mut |buffers| {
            if buffers.os_closed {
                return Some(());
            }
            return None;
        });
    }
}
