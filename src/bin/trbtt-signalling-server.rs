use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_channel as chan;
use futures_util::{future, pin_mut, StreamExt};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tungstenite::{
    handshake::server::{Request, Response},
    Message,
};
use uuid::Uuid;

use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;

use datachannel::{IceCandidate, SessionDescription};

#[derive(Debug, Serialize, Deserialize)]
struct PeerMsg {
    dest_id: Uuid,
    kind: MsgKind,
}

#[derive(Debug, Serialize, Deserialize)]
enum MsgKind {
    Description(SessionDescription),
    Candidate(IceCandidate),
}

// Server part

type PeerMap = Arc<Mutex<HashMap<Uuid, chan::Sender<Message>>>>;

async fn run_server() {
    let peers = PeerMap::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("0.0.0.0:48749")
        .await
        .expect("Listener binding failed");

    while let Ok((stream, _)) = listener.accept().await {
        spawn(handle_new_peer(peers.clone(), stream));
    }
}

async fn handle_new_peer(peers: PeerMap, stream: TcpStream) {
    let mut peer_id = None;

    let callback = |req: &Request, mut resp: Response| {
        let path = req.uri().path();
        let tokens = path.split('/').collect::<Vec<_>>();
        match Uuid::parse_str(tokens[1]) {
            Ok(uuid) => peer_id = Some(uuid),
            Err(err) => {
                log::error!("Invalid uuid: {}", err);
                *resp.status_mut() = StatusCode::BAD_REQUEST;
            }
        }
        Ok(resp)
    };

    let websocket = match tokio_tungstenite::accept_hdr_async(stream, callback).await {
        Ok(websocket) => websocket,
        Err(err) => {
            log::error!("WebSocket handshake failed: {}", err);
            return;
        }
    };

    let peer_id = match peer_id {
        None => return,
        Some(peer_id) => peer_id,
    };
    log::info!("Peer {} connected", &peer_id);

    let (outgoing, mut incoming) = websocket.split();
    let (tx_ws, rx_ws) = chan::unbounded();

    peers.lock().unwrap().insert(peer_id, tx_ws);

    let reply = rx_ws.map(Ok).forward(outgoing);

    let dispatch = async {
        while let Some(Ok(msg)) = incoming.next().await {
            if !msg.is_binary() {
                continue;
            }

            let mut peer_msg = match serde_json::from_slice::<PeerMsg>(&msg.into_data()) {
                Ok(peer_msg) => peer_msg,
                Err(err) => {
                    log::error!("Invalid PeerMsg: {}", err);
                    continue;
                }
            };
            log::info!("Peer {} << {:?}", &peer_id, &peer_msg);

            let dest_id = peer_msg.dest_id;

            match peers.lock().unwrap().get_mut(&dest_id) {
                Some(dest_peer) => {
                    peer_msg.dest_id = peer_id;
                    log::info!("Peer {} >> {:?}", &dest_id, &peer_msg);
                    let peer_msg = serde_json::to_vec(&peer_msg).unwrap();
                    dest_peer.try_send(Message::binary(peer_msg)).ok();
                }
                _ => log::warn!("Peer {} not found in server", &dest_id),
            }
        }
    };

    pin_mut!(dispatch, reply);
    future::select(dispatch, reply).await;

    log::info!("Peer {} disconnected", &peer_id);
    peers.lock().unwrap().remove(&peer_id);
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info");
    let _ = pretty_env_logger::try_init();
    run_server().await
}
