use std::env;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

fn tail_file(file_path: String) -> io::Result<BufReader<std::fs::File>> {
    let f = File::open(file_path)?;
    let mut reader = BufReader::new(f);
    let mut buf = String::new();
    let mut line = reader.read_line(&mut buf);

    loop {
        match line {
            Ok(size) => {
                if size > 0 {
                    let _ = io::stdout().write(buf.as_bytes());
                    buf = String::new();
                    line = reader.read_line(&mut buf);
                }
                else {
                    break;
                }
            },
            Err(_) => break,
        }
    }

    return Ok(reader)
}

fn main() {
    let mut args = env::args();
    let _ = args.next();
    for argument in args {
        let _ = tail_file( argument );
    }
}
