# basic_tcp_proxy

basic_tcp_proxy is a simple crate to launch a TCP proxy on new threads,
redirecting TCP traffic from the listener port to the proxy destination.

This crate is deliberately synchronous for simplicity.

## Example

Example forwarding 127.0.0.1:2000 to 127.0.0.1:4000

```rust
use basic_tcp_proxy::TcpProxy;

fn main() {
    let proxy = TcpProxy::new(2000, "127.0.0.1:4000".parse().unwrap());
    loop {}
}
```
