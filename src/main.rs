mod bgop;
mod wns;
mod stream;

use bgop::BackgroundOp;
use std::env;
use std::ffi::OsStr;
use std::io::BufRead;
use std::io::BufReader;
use std::io::LineWriter;
use std::io::Write;
use std::io;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::thread;
use stream::Line;
use stream::Stream;

fn main() {
    let os = StdoutStream::new();
    let mut os = ProcessStream::new(Box::new(os), env::args().skip(1));
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        eprintln!("[main] Input line: {}", line);
        os.write_line(Arc::from(line));
        if os.rclosed() {
            eprintln!("[main] got rclosed");
            break;
        }
    }
    os.close();
}

struct StdoutStream {
    rclosed: bool,
}

impl StdoutStream {
    fn new() -> StdoutStream {
        return StdoutStream {
            rclosed: false,
        };
    }
}

impl StdoutStream {
    fn maybe_rclosed<T, E>(&mut self, r: Result<T, E>) {
        match r {
            Err(_) => {
                self.rclosed = true;
            }
            Ok(_) => {
            }
        }
    }
}

impl Stream for StdoutStream {
    fn write_line(&mut self, line: Line) {
        self.maybe_rclosed(writeln!(io::stdout(), "{}", line));
    }

    fn rclosed(&mut self) -> bool {
        return self.rclosed;
    }

    fn close(&mut self) {
        // This seems to be all we can do?  We hope/expect the process to be
        // donezo soon anyway...
        self.maybe_rclosed(io::stdout().flush());
    }
}

struct ProcessStream {
    os: Box<Stream>,
    p: Child,
    bgop: Arc<BackgroundOp<Line>>,
}

impl ProcessStream {
    fn new<I, S>(os: Box<Stream>, args: I) -> ProcessStream where I: IntoIterator<Item = S>, S: AsRef<OsStr> {
        let mut args = args.into_iter();
        let mut p = Command::new(args.next().unwrap())
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let bgop = Arc::new(BackgroundOp::<Line>::new());
        {
            let p_stdin = p.stdin.take().unwrap();
            let bgop = bgop.clone();
            thread::spawn(move|| {
                let mut r = LineWriter::new(p_stdin);
                loop {
                    match bgop.be_read_line() {
                        Some(line) => {
                            eprintln!("[backend stdin] got line {}", line);
                            match writeln!(r, "{}", line) {
                                Err(_) => {
                                    eprintln!("[backend stdin] got rclosed");
                                    bgop.be_rclose();
                                }
                                Ok(_) => {
                                }
                            }
                        }
                        None => {
                            eprintln!("[backend stdin] got eof");
                            // drops r
                            return;
                        }
                    }
                }
            });
        }

        {
            let p_stdout = p.stdout.take().unwrap();
            let bgop = bgop.clone();
            thread::spawn(move|| {
                let r = BufReader::new(p_stdout);
                for line in r.lines() {
                    let line = line.unwrap();
                    if !bgop.be_write_line(Arc::from(line)) {
                        eprintln!("[backend stdout] got rclosed");
                        break;
                    }
                }
                bgop.be_close();
                // return drops r
            });
        }

        return ProcessStream {
            os: os,
            p: p,
            bgop: bgop,
        };
    }
}

fn write_on_maybe_line(os: &mut Box<Stream>, bgop: &BackgroundOp<Line>, maybe_line: Option<Line>) {
    match maybe_line {
        Some(line) => {
            os.write_line(line);
            if os.rclosed() {
                bgop.fe_rclose();
            }
        }
        None => {
            os.close();
        }
    }
}

impl Stream for ProcessStream {
    fn write_line(&mut self, line: Line) {
        let os = &mut self.os;
        let bgop = &self.bgop;
        self.bgop.fe_write_line(line, &mut |x| write_on_maybe_line(os, bgop, x));
    }

    fn rclosed(&mut self) -> bool {
        return self.bgop.fe_rclosed();
    }

    fn close(&mut self) {
        let os = &mut self.os;
        let bgop = &self.bgop;
        self.bgop.fe_close(&mut |x| write_on_maybe_line(os, bgop, x));
        self.p.wait().unwrap();
    }
}
