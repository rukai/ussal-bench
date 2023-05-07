use axum::extract::ws::WebSocket;
use tokio::sync::{mpsc, oneshot};

struct Connection {
    socket: WebSocket,
    machine_type: String,
}

#[derive(Debug)]
pub struct Request {
    pub tx: oneshot::Sender<WebSocket>,
    pub machine_type: String,
}

pub async fn task(mut request_rx: mpsc::UnboundedReceiver<Request>) {
    // The order of elements is important! This vec forms a FIFO and requests from the beginning are favored over later events.
    // This ensures that we complete benches from users that submitted first without getting distracted with benches that were submitted later on
    let mut waiting_requests: Vec<Request> = vec![];
    let mut waiting_connections: Vec<Connection> = vec![];

    loop {
        // TODO: select! with incoming connections
        waiting_requests.push(request_rx.recv().await.unwrap());

        if let Some((connection_i, request_i)) = find_match(&waiting_connections, &waiting_requests)
        {
            let connection = waiting_connections.remove(connection_i);
            let request = waiting_requests.remove(request_i);
            request.tx.send(connection.socket).unwrap();
        }
    }
}

fn find_match(connections: &[Connection], requests: &[Request]) -> Option<(usize, usize)> {
    for (connection_i, connection) in connections.iter().enumerate() {
        for (request_i, request) in requests.iter().enumerate() {
            if request.machine_type == connection.machine_type {
                return Some((connection_i, request_i));
            }
        }
    }
    None
}
