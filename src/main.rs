use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

fn main() {
    // open up a TcpListener
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    // notice the unwrap(): bind Returns a Result<TcpListner, Err> type
    // we want to unwrap() the TcpListener or crash trying
    // https://doc.rust-lang.org/std/result/

    // we will receive a TcpStream for every new connection that is opened
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        // pass the stream off to our handler
        handle_connection(stream);
    }

}

fn handle_connection(mut stream: TcpStream) {
    // read the input stream into a buffer
    // http server is dumb and doesn't do anything with it
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();


    // read an html file to respond with
    let contents = fs::read_to_string("hello.html").unwrap();

    // build the HTTP response
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        contents.len(),
        contents
    );

    // write response to the stream
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
}
