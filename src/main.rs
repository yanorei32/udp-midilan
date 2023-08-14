#![warn(clippy::pedantic)]

use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::net::{Ipv4Addr, SocketAddr};

use clap::Parser;
use midir::{MidiIO, MidiInput, MidiOutput};
use tokio::{net::UdpSocket, sync::mpsc, time};

mod model;

type MidiIntr = (mpsc::Receiver<Vec<u8>>, mpsc::Sender<Vec<u8>>);
static PREFIX_MIDI_MESSAGE: u8 = 0x10u8;
static KEEPALIVE_MIDI_MESSAGE: [u8; 4] = [0xf0, 0x73, 0x02, 0xf7];

async fn client(mut midi: MidiIntr, server: SocketAddr) -> Result<(), Box<dyn Error>> {
    let sock = UdpSocket::bind("0.0.0.0:0").await?;
    sock.connect(server).await?;
    sock.send(&KEEPALIVE_MIDI_MESSAGE).await?;

    let mut message = vec![0u8; 128];
    let mut to_send = vec![PREFIX_MIDI_MESSAGE];

    println!("\nClient Ready.");


    loop {
        tokio::select! {
            d = midi.0.recv() => {
                let d = d.ok_or("Failed to recv midi")?;
                println!("<MIDI> Send: {d:?}");
                to_send.splice(1.., d);
                sock.send(&to_send).await?;
            }
            d = sock.recv(&mut message) => {
                let d = d?;

                match (&message[0], &message[1..d]) {
                    (prefix, content) if prefix == &PREFIX_MIDI_MESSAGE => {
                        println!("<MIDI> Recv: {:?}", content);
                        midi.1.send(content.to_vec()).await?;
                    }
                    _ => {
                        println!("<System> Unknown Message: {:?}", &message[1..d]);
                        continue;
                    }
                }

            }
            _ = time::sleep(time::Duration::from_secs(60)) => {
                println!("KeepAlive: {:?}", &KEEPALIVE_MIDI_MESSAGE);
                sock.send(&KEEPALIVE_MIDI_MESSAGE).await?;
            }
        }
    }
}

async fn server(mut midi: MidiIntr, port: u16) -> Result<(), Box<dyn Error>> {
    let listen = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), port);
    let sock = UdpSocket::bind(listen).await?;

    let mut client: Option<SocketAddr> = None;
    let mut message = vec![0u8; 128];
    let mut to_send = vec![PREFIX_MIDI_MESSAGE];

    println!("\nServer Ready.");

    loop {
        tokio::select! {
            d = midi.0.recv() => {
                let Some(client) = client else {
                    continue;
                };

                let d = d.ok_or("Failed to recv midi")?;
                println!("<MIDI> Send: {d:?}");
                to_send.splice(1.., d);
                sock.send_to(&to_send, client).await?;
            },
            d = sock.recv_from(&mut message) => {
                let (len, addr) = d?;

                match (&message[0], &message[1..len]) {
                    (prefix, content) if prefix == &PREFIX_MIDI_MESSAGE => {
                        println!("<MIDI> Recv: {:?}", content);
                        midi.1.send(content.to_vec()).await?;
                    }
                    _ => {
                        println!("<System> Unknown Message: {:?}", &message[1..len]);
                    }
                }

                if client != Some(addr) {
                    println!("Client Connected: {addr}");
                    client = Some(addr);
                }
            },
        }
    }
}

fn ask_port<T: MidiIO>(io: &T, type_str: &str) -> Option<T::Port> {
    let ports = io.ports();

    if ports.is_empty() {
        println!("Failed to find MIDI {type_str} port");
        return None;
    }

    println!("0: DISABLED");

    for (i, p) in ports.iter().enumerate() {
        println!("{}: {}", i + 1, io.port_name(p).unwrap());
    }

    print!("Please select {type_str} port: ");
    stdout().flush().unwrap();

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    match input.trim().parse::<usize>().unwrap() {
        0 => None,
        n => Some(
            ports
                .get(n - 1)
                .ok_or("invalid {type_str} port selected")
                .unwrap()
                .clone(),
        ),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let c: model::Cli = model::Cli::parse();

    let midi_in = MidiInput::new("UDP MidiLAN Input")?;
    println!("\n[Setup] MIDI-In Device --> NETWORK");
    let midi_in_port = ask_port(&midi_in, "Input");
    let (in_tx, in_rx) = mpsc::channel(32);

    if let Some(midi_in_port) = midi_in_port {
        let conn_in = midi_in.connect(
            &midi_in_port,
            "UDP MidiLAN Input",
            move |_, mes, _| {
                in_tx.blocking_send(mes.to_vec()).unwrap();
            },
            (),
        )?;

        std::mem::forget(conn_in);
    }

    let midi_out = MidiOutput::new("UDP MidiLAN Output")?;
    println!("\n[Setup] NETWORK --> MIDI-Out Device");
    let midi_out_port = ask_port(&midi_out, "Output");
    let (out_tx, mut out_rx) = mpsc::channel::<Vec<u8>>(32);

    if let Some(midi_out_port) = midi_out_port {
        let mut conn_out = midi_out.connect(&midi_out_port, "UDP MidiLAN Output")?;
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    v = out_rx.recv() => {
                        conn_out.send(&v.unwrap().clone()).unwrap();
                    }
                }
            }
        });
    }

    match c {
        model::Cli::Server { port } => {
            server((in_rx, out_tx), port).await?;
        }
        model::Cli::Client { host } => {
            client((in_rx, out_tx), host).await?;
        }
    }

    Ok(())
}
