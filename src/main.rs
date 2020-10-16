use std::fs;
use std::thread;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::time::Duration;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

struct ThreadPool {
    threads: Vec<Worker>,
    sender: mpsc::Sender<Job>
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let mut threads = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            Worker::new(id, Arc::clone(&receiver));
        }

        ThreadPool { threads, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F : FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }

}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move ||loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            println!("worker {} recieved job", id);
            job();
        });

        Worker { id, thread }
    }
}

fn main() {
    // open up a TcpListener
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    // notice the unwrap(): bind Returns a Result<TcpListner, Err> type
    // we want to unwrap() the TcpListener or crash trying
    // https://doc.rust-lang.org/std/result/

    let pool = ThreadPool::new(4);

    // we will receive a TcpStream for every new connection that is opened
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        // pass the stream off to our handler

        pool.execute(||{
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    // read the input stream into a buffer
    // http server is dumb and doesn't do anything with it
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    // read an html file to respond with
    let contents = fs::read_to_string("hello.html").unwrap();

    // lets pretend we have a slow connection
    thread::sleep(Duration::from_secs(3));

    // build the HTTP response
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        contents.len(),
        contents
    );

    // write response to the stream
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
