use std::{collections::VecDeque, net::SocketAddr, sync::Mutex};
use once_cell::sync::Lazy;
use speedy::{Readable, Writable};
use tokio::{fs, io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener, sync::mpsc::{unbounded_channel, UnboundedSender}};

#[derive(Readable, Writable)]
struct Data {
	addr: String,
	payload: Vec<u8> 
}

impl Data {
	fn new(addr: SocketAddr, payload: Vec<u8>) -> Data {
		return Data {
			addr: format!("{}", addr),
			payload
		}
	}
}

static BUFFER : Lazy<Mutex<VecDeque<Data>>> = Lazy::new(|| {
	return Mutex::new(VecDeque::new());
});


#[tokio::main]
async fn main() {
	let args : Vec<String> = std::env::args().collect();
	if args.len() < 3 {
		eprintln!("ports not specified");
		eprintln!("Usage: sink.exe [start-port] [end-port]");
	}
	let start_port: u32 = args[1].parse().unwrap();
	let end_port: u32 = args[2].parse().unwrap();

	let (tx, mut rx) = unbounded_channel::<u32>();

	tokio::spawn(async move {
		while let Some(_) = rx.recv().await {
			let local_buf: Vec<Data>;
			{
				let mut guard = BUFFER.lock().unwrap();
				local_buf = guard.drain(0..500).collect::<Vec<Data>>();
			}
			let mut buf: Vec<u8> = vec![];
			let _  = local_buf.iter().map(|d| d.write_to_buffer(&mut buf).unwrap()).collect::<Vec<_>>();
			let mut file = fs::OpenOptions::new().append(true).open("./data.bin").await.unwrap();
			file.write(&buf).await.unwrap();
		}
	});

	for i in start_port..=end_port {
		let t = tx.clone();
		tokio::task::spawn(async move {
			listen(i, t).await;
		});
	}
}
async fn listen(port: u32, tx: UnboundedSender<u32>) {
	let addr = format!("0.0.0.0:{port}");
	let listener = TcpListener::bind(addr).await.expect(&format!("failed to listen on port : {port}"));
	let count = 0;

	match listener.accept().await {
		Ok((mut socket, addr)) => {
			if count > 25 {
				return;
			}
			println!("[INFO] Connection on port: {} from {}", port, addr);
			let mut buf = [0u8; 1024];
			match socket.read(&mut buf[..]).await {
				Ok(n) => {
					println!("[INFO] Read {} bytes from {} on port {}", n, addr, port);
					let data = Data::new(addr, buf[0..n].to_vec());
					{
						let mut guard = BUFFER.lock().unwrap();
						guard.push_back(data);
						if guard.len() > 500 {
							tx.send(1).unwrap();
						}
					}
				},
				Err(e) => {
					eprintln!("[ERROR] Failed to read from : {} on port {}. e: {}", addr, port, e);
				}
			}
		}
		Err(e) => {
			eprintln!("[ERROR] Failed to accept connection : {}", e);
		}
	}
}