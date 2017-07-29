use std::env;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

fn tail_file(file_path: String) -> io::Result<&BufReader<std::fs::File>> {
    let f = File::open(file_path)?;
    let reader = tail_reader(&mut BufReader::new(f));
    for line in reader.lines() {
        let _ = io::stdout().write(line.unwrap().as_bytes());
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
