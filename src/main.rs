use std::io::{stdout, Write};
use std::net::SocketAddr;

use crossterm::{
    cursor::MoveToColumn,
    execute,
    terminal::{Clear, ClearType},
};

use tokio::time::{sleep, Duration};
use tokio_modbus::client::Context;
use tokio_modbus::prelude::*;

const LOGO_SOCKET: &str = "192.168.39.201:502";


const INPUTS_COUNT: usize = 24;
const OUTPUTS_COUNT: usize = 20;

const LOGO_Q_START: u16 = 8193;

mod flag {
    pub const M1: u16 = 8257;
    pub const M2: u16 = 8258;
    pub const M3: u16 = 8259;
    pub const M4: u16 = 8260;
    pub const M5: u16 = 8261;
    pub const M6: u16 = 8262;
    pub const M7: u16 = 8263;
    pub const M8: u16 = 8264;
    pub const M9: u16 = 8265;
    pub const M10: u16 = 8266;
    pub const M11: u16 = 8267;
    pub const M12: u16 = 8268;
    pub const M13: u16 = 8269;
    pub const M14: u16 = 8270;
    pub const M15: u16 = 8271;
    pub const M16: u16 = 8272;
    pub const M17: u16 = 8273;
    pub const M18: u16 = 8274;
    pub const M19: u16 = 8275;
    pub const M20: u16 = 8276;
}


struct LogoDriver {
    client: Context,
    
    inputs: Vec<bool>,
    outputs: Vec<bool>,
    flags: Vec<bool>,
}

impl LogoDriver {
    async fn reverse_q(&mut self, address: usize,) -> Result<(), Box<dyn std::error::Error>> {
        let value = !self.outputs[address];

        self.client
            .write_single_coil(LOGO_Q_START + address as u16, value)
            .await??;

        self.outputs[address] = value;

        Ok(())
    }
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = LOGO_SOCKET.parse()?;

    let mut client = tcp::connect(addr).await?;


    loop {
        logo.inputs = client
            .read_discrete_inputs(0, logo.inputs.len() as u16)
             .await??;
        
        logo.outputs = client
            .read_coils(LOGO_Q_START, logo.outputs.len() as u16)
            .await??;
        
        let mut out = stdout();

        execute!(
            out,
            MoveToColumn(0),
            Clear(ClearType::CurrentLine)
        )?;

        for value in &logo.inputs {
            print!("{} ", if *value { 1 } else { 0 });
        }

        stdout().flush()?;

        sleep(Duration::from_millis(1)).await;
    }
}
