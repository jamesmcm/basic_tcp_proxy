use std::io::{BufRead, BufReader, Write};
use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr};
use std::net::{TcpListener, TcpStream};
use thiserror::Error;

/// TcpProxy runs one thread looping to accept new connections
/// and then two separate threads per connection for writing to each end
pub struct TcpProxy {
    /// The handle for the outer thread, accepting new connections
    pub forward_thread: std::thread::JoinHandle<()>,
}

impl TcpProxy {
    /// Create a new TCP proxy, binding to listen_port and forwarding and receiving traffic from
    /// proxy_to
    pub fn new(listen_port: u16, proxy_to: SocketAddr) -> Result<Self, ProxyError> {
        let listener_forward = TcpListener::bind(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            listen_port,
        ))?;

        let sender_forward_original = TcpStream::connect(proxy_to)?;
        let sender_backward_original = sender_forward_original.try_clone()?;

        let forward_thread = std::thread::spawn(move || {
            loop {
                let (stream_forward, _addr) = listener_forward
                    .accept()
                    .expect("Failed to accept connection");
                let mut stream_backward =
                    stream_forward.try_clone().expect("Failed to clone stream");

                let mut sender_forward = sender_forward_original.try_clone().unwrap();
                let sender_backward = sender_backward_original.try_clone().unwrap();
                std::thread::spawn(move || {
                    let mut stream_forward = BufReader::new(stream_forward);
                    loop {
                        let length = {
                            let buffer = stream_forward.fill_buf().unwrap();
                            let length = buffer.len();
                            if buffer.is_empty() {
                                // Connection closed
                                return;
                            }
                            sender_forward
                                .write_all(&buffer)
                                .expect("Failed to write to remote");
                            sender_forward.flush().expect("Failed to flush remote");
                            length
                        };
                        stream_forward.consume(length);
                    }
                });

                let _backward_thread = std::thread::spawn(move || {
                    let mut sender_backward = BufReader::new(sender_backward);
                    loop {
                        let length = {
                            let buffer = sender_backward.fill_buf().unwrap();
                            let length = buffer.len();
                            if buffer.is_empty() {
                                // Connection closed
                                return;
                            }
                            if stream_backward.write_all(&buffer).is_err() {
                                // Connection closed
                                return;
                            }

                            stream_backward.flush().expect("Failed to flush locally");
                            length
                        };
                        sender_backward.consume(length);
                    }
                });
            }
        });

        Ok(Self { forward_thread })
    }
}

/// Possible error if we socket address fails to bind (for local connection)
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("Failed to bind to socket address")]
    BindError(#[from] std::io::Error),
}