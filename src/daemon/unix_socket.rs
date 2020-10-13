use super::connection_handler::ConnectionData;

use async_std::{
    io::{prelude::*, BufReader, BufWriter},
    os::unix::net::{UnixListener, UnixStream},
    prelude::*,
    sync::Mutex,
};
use galaxy_buds_live_rs::message::{set_noise_reduction, Payload};

use std::{path::Path, sync::Arc};

/// Runs the unix socket which
/// provides the userspace API
pub async fn run<P: AsRef<Path>>(p: P, cd: Arc<Mutex<ConnectionData>>) {
    let p = p.as_ref();
    let listener = UnixListener::bind(p).await.unwrap();
    let mut incoming = listener.incoming();

    loop {
        for stream in incoming.next().await {
            match stream {
                Ok(stream) => {
                    println!("connected");
                    async_std::task::spawn(handle_client(stream, Arc::clone(&cd)));
                }
                Err(err) => {
                    println!("Error: {}", err);
                    break;
                }
            }
        }
    }
}

/// Handle unix socket connections
async fn handle_client(stream: UnixStream, cd: Arc<Mutex<ConnectionData>>) {
    let mut read_stream = BufReader::new(&stream);
    let mut write_stream = BufWriter::new(&stream);
    let mut buff = String::new();

    loop {
        buff.clear();

        read_stream.read_line(&mut buff).await.unwrap();
        let locked = cd.lock().await;
        let info = locked.get_first_device().unwrap();

        if buff == "a\n" {
            let mut v = locked.get_first_stream();
            let send_msg = set_noise_reduction::new(true);
            v.write(&send_msg.to_byte_array()).await.unwrap();
            continue;
        }

        let v = info;
        write_stream
            .write(format!("{:?}", v).as_bytes())
            .await
            .unwrap();

        if let Err(_) = write_stream.flush().await {
            return;
        }
    }
}
