use std::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_modbus::prelude::*;

#[tokio::test]
async fn reads_discrete_inputs_from_logo_simulator() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await?;

        let mut request = [0u8; 12];
        stream.read_exact(&mut request).await?;

        assert_eq!(request[2..4], [0x00, 0x00], "Modbus/TCP protocol id must be 0");
        assert_eq!(request[7], 0x02, "function code must be read discrete inputs");

        let expected_start = u16::from_be_bytes([request[8], request[9]]);
        let expected_quantity = u16::from_be_bytes([request[10], request[11]]);
        assert_eq!(expected_start, 0, "start address must be 0");
        assert_eq!(expected_quantity, 8, "we read 8 discrete inputs");

        let bits = [true, false, true, true, false, false, true, false];
        let mut byte = 0u8;
        for (index, value) in bits.iter().enumerate() {
            if *value {
                byte |= 1u8 << index;
            }
        }

        let response = [
            0x00, 0x00, // transaction id (must match request)
            0x00, 0x00, // protocol id
            0x00, 0x04, // length = unit id + function + byte count + data
            0xFF,       // unit id
            0x02,       // function code
            0x01,       // byte count
            byte,       // data
        ];

        stream.write_all(&response).await?;
        stream.flush().await?;

        Ok::<_, io::Error>(())
    });

    let mut client = tcp::connect(addr).await?;
    let values = client.read_discrete_inputs(0, 8).await??;

    server.await??;

    assert_eq!(values, vec![true, false, true, true, false, false, true, false]);
    Ok(())
}
