#![allow(non_snake_case, unused)]

use std::{net::TcpListener, io::{prelude::*, BufReader}};
use tokio::{task::spawn_blocking};

struct TcpServer { }

impl TcpServer {
	// new starts a multi-threaded TCP server.
	fn new( ) {
		println!("INFO: 🚀 Starting TCP server");

		let tcpLister= TcpListener::bind("127.0.0.1:6379")
			.expect("ERROR: binding the TCP server to given socket address");

		for stream in tcpLister.incoming( ) {
			let mut stream= stream.expect("ERROR: parsing incoming connection stream");

			spawn_blocking(move | | {
				let bufferReader= BufReader::new(&stream);
				let request: Vec<_> = bufferReader
																.lines( )
																.map(|result| result.expect("ERROR: parsing line in incoming connection stream"))
																.take_while(|line| !line.is_empty( ))
																.collect( );
			});
		}
	}
}

fn main( ) {
	TcpServer::new( );
}