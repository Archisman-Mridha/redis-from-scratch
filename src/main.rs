#![allow(non_snake_case)]

// To communicate with the Redis server, Redis clients use a protocol called REdis Serialization
// Protocol (RESP).
// Read more here - https://redis.io/docs/reference/protocol-spec.
mod resp {

	#[derive(PartialEq, Debug)]
	pub enum RespType<'a> {

		// Simple strings are encoded as a plus (+) character, followed by a string. The string mustn't
		// contain a CR (\r) or LF (\n) character and is terminated by CRLF (i.e., \r\n).
		SimpleString(&'a str),

		// A bulk string represents a single binary string.
		// Read more here - https://redis.io/docs/reference/protocol-spec/#bulk-strings.
		// Format of data it holds - One or more decimal digits (0..9) as the string's length, in bytes,
		// as an unsigned, base-10 value..
		BulkString(&'a str),

		// Represents an array of RESP encoded elements.
		// Read more here - https://redis.io/docs/reference/protocol-spec/#arrays.
		Array(Vec<Self>)
	}

	pub fn encode<'respType, 's>(respType: RespType<'respType>) -> String {
		match respType {

			RespType::SimpleString(s) => format!("+{}\r\n", s),

			RespType::BulkString(s) => format!("${}\r\n{}\r\n", s.len( ), s),

			RespType::Array(array) => {
				let mut result= String::from(format!("*{}\r\n", array.len( )));

				for item in array {
					result.push_str(&encode(item));}

				result
			}
		}
	}

	// decode takes in a RESP encoded string, parses it and returns the result. The RESP encoded string
	// can contain a single element or multiple elements concatenated together.
	pub fn decode<'a>(s: &'_ str) -> (RespType, &'_ str) {
		let firstcharacter= s.chars( ).next( ).unwrap( );
		match firstcharacter {

			'+' => {
				let s= s.strip_prefix('+').unwrap( );

				let (decodedValue, remaining)= s.split_once("\r\n").unwrap( );

				(RespType::SimpleString(decodedValue), remaining)
			},

			'$' => {
				let (length, remaining)= s.strip_prefix('$').unwrap( )
													.split_once("\r\n").unwrap( );

				let (s, remaining)= remaining.split_once("\r\n").unwrap( );

				let length= length.parse::<usize>( ).unwrap( );
				if s.len( ) != length {
					panic!( )}

				(RespType::BulkString(s), remaining)
			},

			'*' => {
				let (length, mut data)= s.strip_prefix('*').unwrap( )
																 .split_once("\r\n").unwrap( );

				let length= length.parse::<usize>( ).unwrap( );
				let mut vector= Vec::<RespType>::with_capacity(length);

				for _ in 0..length {
					let (respType, remaining)= decode(data);

					data= remaining;
					vector.push(respType);
				}

				let remaining= data;
				(RespType::Array(vector), remaining)
			},

			_ => panic!( )
		}
	}
}

mod server {
	use std::{net::TcpListener, io::{prelude::*, BufReader}, thread::spawn};
	use crate::resp::{self, RespType};

	fn pingHandler<'a>(arg: &'_ str) -> RespType<'_> {
		RespType::SimpleString(arg)
	}

	pub struct TcpServer;

	impl TcpServer {
		// new starts a multi-threaded TCP server.
		pub fn new( ) {
			println!("INFO: 🚀 Starting TCP server");

			let tcpLister= TcpListener::bind("127.0.0.1:6379")
				.expect("ERROR: binding the TCP server to given socket address");

			for stream in tcpLister.incoming( ) {
				let mut stream= stream.expect("ERROR: parsing incoming connection stream");

				spawn(move | | {
					let mut bufferReader= BufReader::new(&stream);

					let mut request= String::new( );
					bufferReader.read_to_string(&mut request)
											.expect("ERROR: parsing incoming connection stream");

					let response= Self::handleRequest(&request);

					stream.write_all(response.as_bytes( )).unwrap( );
				});
			}
		}

		// handleRequest takes in a request, parses it to a Redis command, executes that command and
		// returns RESP encoded execution result.
		pub fn handleRequest<'a>(request: &str) -> String {
			// Redis generally uses RESP as a request-response protocol in the following way :
			//
			// 1. Clients send commands to a Redis server as an array of bulk strings. The first (and
			//		sometimes also the second) bulk string in the array is the command's name. Subsequent
			// 		elements of the array are the arguments for the command.
			//
			// 2. The server replies with a RESP type.

			let firstCharacter= request.chars( ).next( ).unwrap( );
			if firstCharacter != '*' {
				panic!( )}

			let (decodedRequest, _)= resp::decode(&request);

			let decodedResponse: RespType= match decodedRequest {
				RespType::Array(array) => {
					let mut vector= Vec::<&'a str>::with_capacity(array.len( ));

					for respType in array {
						match respType {
							RespType::BulkString(s) => vector.push(s),

							_ => panic!( )
						}
					}

					Self::handleCommand(vector)
				},
				_ => unreachable!( )
			};

			resp::encode(decodedResponse)
		}

		// handleCommand takes in a Redis command and executes it. The execution result is returned.
		fn handleCommand(vector: Vec<&str>) -> RespType {
			let mut iterator= vector.iter( );

			let command= *iterator.next( ).unwrap( );
			match command {

				// The 'PING' command returns 'PONG' if no argument is provided, otherwise returns a copy of
				// the argument as a bulk.
				"PING" => {
					let arg= iterator.next( )
													 .map_or("PONG", |v| *v);

					pingHandler(arg)
				},

				"SET" => todo!( ),

				"GET" => todo!( ),

				_ => panic!( )
			}
		}
	}
}

mod client {
	use std::{io::{stdin, stdout, Write}, net::{TcpStream, SocketAddr}, time::Duration};

	pub struct Client {
		connection: TcpStream
	}

	impl Client {
		pub fn new(serverAddress: &str) -> Self {
			let serverAddress= serverAddress.parse::<SocketAddr>( )
																			.expect("ERROR: Invalid Redis server address received");

			let connection= TcpStream::connect_timeout(&serverAddress, Duration::from_secs(3))
																 .expect("ERROR: Couldn't connect to the Redis server");

			Client { connection }
		}

		// startRepl starts an interactive REPL where the user can execute Redis commands at a time.
		pub fn startRepl(&mut self) {
			loop {
				print!(">> ");
				stdout( ).flush( ).unwrap( );

				let mut input= String::new( );
				stdin( ).read_line(&mut input).unwrap( );

				todo!( )
			}
		}
	}
}

fn main( ) {
	let request= "*1\r\n$4\r\nPING\r\n";

	let result= server::TcpServer::handleRequest(request);
	println!("{}", result);
}