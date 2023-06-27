use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, info_span, Instrument};

use thiserror::Error;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Error)]
pub enum RoomError {
    #[error("User failed to join the room")]
    JoinError(#[from] io::Error),
}

/// Room are multiple users chatting with eachother.
/// Technically it's hodling a websocket connection to each of the users,
/// and broadcasts any message sent within the room.

#[derive(Debug)]
pub struct Room {
    peer_map: Arc<Mutex<HashMap<SocketAddr, UnboundedSender<Message>>>>,
    listener: TcpListener,
}

impl Room {
    pub fn new(listener: TcpListener) -> Room {
        Room {
            peer_map: Arc::new(Mutex::new(HashMap::new())),
            listener,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn run(&mut self) -> Result<(), RoomError> {
        loop {
            let (stream, addr) = self
                .listener
                .accept()
                .instrument(info_span!("accept"))
                .await?;
            info!(%addr, "New user from {addr} incoming");
            let mut peers = self.peer_map.clone();
            tokio::spawn(handle_user(peers, stream, addr));
        }
    }
}

#[tracing::instrument(skip(stream, peer_map))]
async fn handle_user(
    mut peer_map: Arc<Mutex<HashMap<SocketAddr, UnboundedSender<Message>>>>,
    stream: TcpStream,
    addr: SocketAddr,
) {
    info!(%addr, "user joins the room");
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("failed to accept stream");

    let (mut tx, rx) = unbounded();
    peer_map
        .lock()
        .expect("failed to obtain peer map mutex!")
        .insert(addr, tx);

    let (mut outgoing, mut incoming) = ws_stream.split();
    let message_incoming = incoming
        .try_for_each(|msg| {
            match msg {
                Message::Text(ref txt) => {
                    info!(%addr, msg = ?txt, "message ");
                    // fanout message to others in the room.
                    // We could filter original sender, but whatever. Pretend sending message back is a confirmation.
                    for peer in peer_map.lock().unwrap().iter_mut() {
                        // This lock should be probably
                        info!(%addr, "peer map mutext lock obtained");
                        let peer_addr = peer.0;
                        let tx = peer.1;
                        info!(%addr, recipient = %peer_addr, "room fanout");
                        let msg = msg.clone();
                        tx.unbounded_send(msg).unwrap();
                    }
                }
                Message::Binary(_) => {}
                Message::Ping(_) => {}
                Message::Pong(_) => {}
                Message::Close(_) => {
                    info!(%addr, "user leaving the room, removing from peer map");
                    peer_map.lock().unwrap().remove(&addr);
                }
                Message::Frame(_) => {}
            }
            future::ok(())
        })
        .instrument(info_span!("message incoming"));
    let broadcast_message = rx.map(Ok).forward(outgoing);
    future::select(message_incoming, broadcast_message).await;
    info!(%addr, "user disconnected");
}
