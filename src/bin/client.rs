use basic_tcp_proxy::TcpProxy;

fn main() {
    let proxy = TcpProxy::new(2000, "127.0.0.1:4000".parse().unwrap());
    loop {}
}
