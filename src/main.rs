use std::error::Error;
use std::io;
use std::net::SocketAddr;

use tokio::sync::{mpsc, watch};
use tokio::time::{self, Duration};
use tokio_modbus::client::Context;
use tokio_modbus::prelude::*;

mod view;

const LOGO_SOCKET: &str = "192.168.1.201:502";
const LOGO_UNIT_ID: u8 = 255;
const INPUTS_COUNT: usize = 24;
const OUTPUTS_COUNT: usize = 20;
const LOGO_Q_START: u16 = 8192;
const POLL_INTERVAL: Duration = Duration::from_millis(20);
const IMPULSE_DURATION: Duration = Duration::from_millis(100);
const RECONNECT_DELAY: Duration = Duration::from_secs(1);

async fn run_driver(
    addr: SocketAddr,
    state_tx: watch::Sender<view::IoState>,
    mut command_rx: mpsc::Receiver<view::OutputCommand>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    loop {
        let mut client = match tcp::connect_slave(addr, Slave(LOGO_UNIT_ID)).await {
            Ok(client) => client,
            Err(error) => {
                let error = contextual_error(format!("TCP connect to {addr} failed"), error);
                state_tx.send_modify(|state| {
                    state.status = format!("Connection error: {error}");
                });
                time::sleep(RECONNECT_DELAY).await;
                continue;
            }
        };

        match run_session(&mut client, &state_tx, &mut command_rx).await {
            Ok(()) => return Ok(()),
            Err(error) => {
                state_tx.send_modify(|state| {
                    state.status = format!("Disconnected: {error}; reconnecting...");
                });
                time::sleep(RECONNECT_DELAY).await;
            }
        }
    }
}

async fn run_session(
    client: &mut Context,
    state_tx: &watch::Sender<view::IoState>,
    command_rx: &mut mpsc::Receiver<view::OutputCommand>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut poll = time::interval_at(time::Instant::now() + POLL_INTERVAL, POLL_INTERVAL);
    poll.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            command = command_rx.recv() => {
                let Some(command) = command else {
                    return Ok(());
                };

                let (address, value, is_impulse) = match command {
                    view::OutputCommand::Set { address } => (address, true, false),
                    view::OutputCommand::Reset { address } => (address, false, false),
                    view::OutputCommand::Impulse { address } => (address, true, true),
                };
                write_output(client, address, value).await?;
                write_output(client, address, value).await?;
                state_tx.send_modify(|state| {
                    state.outputs[address] = value;
                });

                if is_impulse {
                    time::sleep(IMPULSE_DURATION).await;
                    write_output(client, address, false).await?;
                    state_tx.send_modify(|state| {
                        state.outputs[address] = false;
                    });
                }
            }
            _ = poll.tick() => {
                let inputs = client
                    .read_discrete_inputs(0, INPUTS_COUNT as u16)
                    .await
                    .map_err(|error| contextual_error(format!("Transport error reading discrete inputs (address 0, count {INPUTS_COUNT}, Unit ID {LOGO_UNIT_ID})"), error))?
                    .map_err(|error| contextual_error(format!("Modbus error reading discrete inputs (address 0, count {INPUTS_COUNT}, Unit ID {LOGO_UNIT_ID})"), error))?;

                state_tx.send_modify(|state| {
                    state.inputs = inputs;
                    state.status = "Connected".to_owned();
                });

                let outputs = client
                    .read_coils(LOGO_Q_START, OUTPUTS_COUNT as u16)
                    .await
                    .map_err(|error| contextual_error(format!("Transport error reading coils (address {LOGO_Q_START}, count {OUTPUTS_COUNT})"), error))?
                    .map_err(|error| contextual_error(format!("Modbus error reading coils (address {LOGO_Q_START}, count {OUTPUTS_COUNT})"), error))?;

                state_tx.send_modify(|state| {
                    state.outputs = outputs;
                });
            }
        }
    }
}

fn contextual_error(stage: impl std::fmt::Display, error: impl std::fmt::Debug) -> io::Error {
    io::Error::other(format!("{stage}: {error:?}"))
}

async fn write_output(
    client: &mut Context,
    address: usize,
    value: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    client
        .write_single_coil(LOGO_Q_START + address as u16, value)
        .await
        .map_err(|error| {
            contextual_error(
                format!(
                    "Transport error writing Q{} (Modbus address {})",
                    address + 1,
                    LOGO_Q_START + address as u16
                ),
                error,
            )
        })?
        .map_err(|error| {
            contextual_error(
                format!(
                    "Modbus error writing Q{} (Modbus address {})",
                    address + 1,
                    LOGO_Q_START + address as u16
                ),
                error,
            )
        })?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr: SocketAddr = LOGO_SOCKET.parse()?;
    let initial_state = view::IoState::new(INPUTS_COUNT, OUTPUTS_COUNT);
    let (state_tx, state_rx) = watch::channel(initial_state);
    let (command_tx, command_rx) = mpsc::channel(32);

    let driver_state_tx = state_tx.clone();
    tokio::spawn(async move {
        if let Err(error) = run_driver(addr, driver_state_tx.clone(), command_rx).await {
            driver_state_tx.send_modify(|state| {
                state.status = format!("Modbus error: {error}");
            });
        }
    });

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Siemens LOGO! Modbus",
        options,
        Box::new(move |_creation_context| Ok(Box::new(view::Window::new(state_rx, command_tx)))),
    )?;

    Ok(())
}
