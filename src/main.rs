extern crate notify;
extern crate crc;
extern crate websocket;

use std::env;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::{Path, PathBuf};

use notify::{Watcher, RecommendedWatcher, RecursiveMode, DebouncedEvent};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Duration;

use crc::{crc32};
use websocket::{ClientBuilder, OwnedMessage};

fn tail_file(file_path: PathBuf) ->
    io::Result<(BufReader<std::fs::File>, Sender<String>, Receiver<String>)>
{
    let f = File::open(file_path)?;
    let reader = BufReader::new(f);
    let (line_tx, line_rx) = channel();
    
    return Ok((consume_reader(reader, &line_tx).unwrap(), line_tx, line_rx))
}

fn consume_reader(mut reader: BufReader<std::fs::File>, sender: &Sender<String>) ->
    io::Result<BufReader<std::fs::File>>
{
    let mut buf = String::new();
    let mut line = reader.read_line(&mut buf);
    
    loop {
        match line {
            Ok(size) => {
                if size > 0 {
                    let _ = buf.pop(); //Remove newline character
                    let _ = sender.send(buf);

                    buf = String::new();
                    line = reader.read_line(&mut buf);
                }
                else {
                    break;
                }
            },
            Err(e) => return Err(e)
        }
    }
    
    return Ok(reader)
}

fn start_watching(dir_path: PathBuf) -> io::Result<(RecommendedWatcher, Receiver<DebouncedEvent>)> {
    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(tx, Duration::from_secs(1)).unwrap();

    watcher.watch(dir_path, RecursiveMode::Recursive).unwrap();

    return Ok((watcher, rx))
}

fn verify_checksum(checksum: String, data: String) -> Result<String, ()> {
    let checksum_value = checksum.parse::<u32>().unwrap();
    let computed_checksum = crc32::checksum_ieee(data.as_bytes());

    if checksum_value == computed_checksum {
        return Ok(data)
    }
    else {
        return Err(())
    }
}

fn main() {
    let mut args = env::args();
    let _ = args.next();
    let data_file_path = args.next().unwrap();
    let websocket_address = args.next().unwrap();
    
    let path = Path::new(&data_file_path);
    if path.exists() && path.is_file() {
        let (mut reader, line_tx, line_rx) = tail_file(path.to_path_buf()).unwrap();
        
        let parent = path.parent().unwrap();
        let (watcher, rx) = start_watching(parent.to_path_buf()).unwrap();

        let mut websocket_client = ClientBuilder::new(&websocket_address)
            .unwrap()
            .connect_secure(None)
            .unwrap();
        
        let receiver_thread = thread::spawn( move || {
            loop {
                let crc_string = line_rx.recv().unwrap();
                let data = line_rx.recv().unwrap();
                
                match verify_checksum(crc_string, data) {
                    Ok(s) => println!("Would send {:?}", s),
                    //websocket_client.send_message(&OwnedMessage::Text(s)).unwrap(),
                    Err(_) => println!("Failed checksum"),
                }
            }
        });
        
        loop {
            let event = rx.recv().unwrap();
            
            match event {
                DebouncedEvent::NoticeWrite(p) => {
                    if p == path {
                        reader = consume_reader(reader, &line_tx).unwrap();
                    }
                },
                DebouncedEvent::NoticeRemove(p) => {
                    if p == path {
                        break;
                    }
                },
                DebouncedEvent::Create(_) => (),
                DebouncedEvent::Write(p) => {
                    if p == path {
                        reader = consume_reader(reader, &line_tx).unwrap();
                    }
                },
                DebouncedEvent::Chmod(_) => (),
                DebouncedEvent::Remove(p) => {
                    if p == path {
                        break;
                    }
                },
                DebouncedEvent::Rename(p_old, _) => {
                    if p_old == path {
                        break;
                    }
                },
                DebouncedEvent::Rescan => (),
                DebouncedEvent::Error(_, _) => break,
            }
        }
        
        receiver_thread.join().unwrap();
    }
}
