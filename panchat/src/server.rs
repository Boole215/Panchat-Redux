use std::net::UdpSocket;

fn main() -> std::io::Result<()>{
    {
	let socket = UdpSocket::bind("127.0.0.1:69420")?;

	// Receives a single datagram message on the socket.
	let mut buf = [0; 10];
	let (amt, src) = socket.recv_from(&mut buf)?;

	let buf = &mut buf[..amt];
	buf.reverse();
	socket.send_to(buf, &src)?;
    }
    Ok(());
}
