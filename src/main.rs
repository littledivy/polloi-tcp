mod polloi;

use futures::FutureExt;
use futures::StreamExt;
use polloi::Runtime;
use polloi::TcpListener;
use polloi::TcpStream;
use std::io;
use std::rc::Rc;
use std::time::Duration;

fn main() {
    let runtime = polloi::Runtime::new().expect("new runtime");
    runtime.block_on(async_main(&runtime)).expect("main")
}

async fn async_main(runtime: &Rc<Runtime>) -> io::Result<()> {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let listener = TcpListener::bind(runtime, addr).expect("bind");

    listener
        .set_defer_accept(Duration::from_secs(10))
        .expect("TCP_DEFER_ACCEPT");

    let mut clients = futures::stream::FuturesOrdered::new();

    loop {
        futures::select! {
            r = listener.accept().fuse() => {
                let (socket, _) = r.expect("accept");
                clients.push_back(process_socket(socket));
            },
            r = clients.select_next_some() => {
                if let Err(e) = r {
                    eprintln!("client error: {}", e);
                }
            },
        }
    }
}

async fn process_socket(socket: TcpStream) -> io::Result<()> {
    let mut req = [0; 4096];
    let res = b"HTTP/1.1 200 OK\r\nContent-length: 12\r\n\r\nHello world\n";

    loop {
        let n = socket.read(&mut req).await?;
        if n == 0 {
            return Ok(());
        }
        let mut i = 0;
        while i < res.len() {
            i += socket.write(&res[i..]).await?;
        }
    }
}
