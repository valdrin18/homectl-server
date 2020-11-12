use anyhow::{Context, Result};

use crate::homectl_core::{
    events::{Message, TxEventChannel},
    integration::IntegrationId,
};

use super::{
    utils::{from_lifx_state, read_lifx_msg, LifxMsg},
    LifxConfig, UdpSenderMsg,
};
// use mio::net::UdpSocket;
// use mio::{Events, Interest, Poll, Token};
use async_std::net::UdpSocket;
use std::{sync::Arc, io};
use std::{net::SocketAddr, sync::mpsc::Sender, time::Duration};
use tokio::time::{interval_at, Instant};

const MAX_UDP_PACKET_SIZE: usize = 1 << 16;

pub async fn init_udp_socket(_config: &LifxConfig) -> Result<UdpSocket> {
    // Setup the UDP socket. LIFX uses port 56700.
    let addr: SocketAddr = "0.0.0.0:56700".parse()?;

    let socket: UdpSocket = UdpSocket::bind(addr).await?;
    socket
        .set_broadcast(true)
        .context("set_broadcast call failed")?;

    Ok(socket)
}

pub async fn handle_lifx_msg(msg: LifxMsg, integration_id: IntegrationId, sender: TxEventChannel) {
    match msg {
        LifxMsg::State(state) => {
            let device = from_lifx_state(state, integration_id.clone());
            sender
                .send(Message::IntegrationDeviceRefresh { device })
                .await;
        }
        _ => {}
    }
}

pub fn listen_udp_stream(
    socket: Arc<UdpSocket>,
    integration_id: IntegrationId,
    sender: TxEventChannel,
) {
    let mut buf: [u8; MAX_UDP_PACKET_SIZE] = [0; MAX_UDP_PACKET_SIZE];
    tokio::spawn(async move {
        loop {
            let res = socket.recv_from(&mut buf).await;

            match res {
                // FIXME: should probably do some sanity checks on bytes_read
                Ok((_bytes_read, addr)) => {
                    let msg = read_lifx_msg(&buf, addr);

                    handle_lifx_msg(msg, integration_id.clone(), sender.clone()).await;
                }
                Err(e) => {
                    println!("Error in udp recv_from {}", e);
                }
            }
        }
    });
}

pub async fn poll_lights(udp_sender_tx: Sender<UdpSenderMsg>) -> io::Result<()> {
    let poll_rate = Duration::from_millis(1000);
    let start = Instant::now() + poll_rate;
    let mut interval = interval_at(start, poll_rate);

    // TODO: find and use the subnet broadcast address instead
    let broadcast_addr = "255.255.255.255:56700".parse::<SocketAddr>().unwrap();

    let msg = LifxMsg::Get(broadcast_addr);

    loop {
        interval.tick().await;

        udp_sender_tx.send(msg.clone()).unwrap();
    }
}
