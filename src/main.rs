use std::io::{stdout, Write};
use std::net::SocketAddr;
use std::sync::mpsc::{self, Sender};

use crossterm::{
    cursor::MoveToColumn,
    execute,
    terminal::{Clear, ClearType},
};

use tokio::time::{sleep, Duration};
use tokio_modbus::client::Context;
use tokio_modbus::prelude::*;

mod view;

const LOGO_SOCKET: &str = "192.168.39.201:502";


const INPUTS_COUNT: usize = 24;
const OUTPUTS_COUNT: usize = 20;

const LOGO_Q_START: u16 = 8193;

enum Changes {
    Input {
        address: usize,
        value: bool,
    },
    Output {
        address: usize,
        value: bool,   
    },
}



struct LogoDriver {
    client: Context,
    
    inputs: Vec<bool>,
    outputs: Vec<bool>,

    tx: Sender<Changes>
}

impl LogoDriver {
    fn new(client: Context, tx: Sender<Changes>) -> Self {
        Self {
            client, 
            inputs: vec![false; INPUTS_COUNT],
            outputs: vec![false; OUTPUTS_COUNT],
            tx: tx,   
        }
    }

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

    let (tx, rx) = mpsc::channel::<Changes>(); 
    let mut client = tcp::connect(addr).await?;

    let mut logo = LogoDriver::new(client, tx);

    loop {
        logo.inputs = logo.client
            .read_discrete_inputs(0, logo.inputs.len() as u16)
             .await??;
        
        logo.outputs = logo.client
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

        for value in &logo.outputs {
            print!("{} ", if *value { 1 } else { 0 });
        }
        


        stdout().flush()?;

        sleep(Duration::from_millis(1)).await;
    }
}
