use std::io::{self, Write, Read};
use std::net::{TcpListener, TcpStream};


const LOCALHOST_IP: &str = "0.0.0.0";
const LOCALHOST_PORT: u16 = 502;

const HOST_IP: &str = LOCALHOST_IP;
const HOST_PORT: u16 = 502;


fn handle_client(stream: TcpStream) {
    // ...
} 

pub struct ModbusDriver {
    pub address: String,
    connection: TcpListener,
}

impl ModbusDriver {
    pub fn new(ip: &str, port: u16) -> io::Result<Self> {
        let address = format!("{ip}:{port}");
        let connection = TcpListener::bind(&address)?;

        Ok(Self {
            address,
            connection,
        })
    }

    pub fn run(&self) -> io::Result<()> {
        for stream in self.connection.incoming() {
            match stream {
                Ok(stream) => {
                    let device = Device::new(self, stream);
                    // ...
                },
                Err(err) => {
                    eprintln!("Connection error: {err}");
                }
            }
        }

        Ok(())
    }
}


pub struct Device<'a> {
    pub server: &'a ModbusDriver,
    pub stream: TcpStream,
}

impl<'a> Device<'a> {
    pub fn new(server: &'a ModbusDriver, stream: TcpStream) ->Self {
        Self { server, stream}
    }
}
