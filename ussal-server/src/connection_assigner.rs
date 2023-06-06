use tokio::sync::{mpsc, oneshot};
use ussal_networking::runner_protocol as runner_proto;

#[derive(Debug)]
pub struct Connection {
    pub tx: mpsc::UnboundedSender<runner_proto::JobRequest>,
    pub rx: mpsc::UnboundedReceiver<runner_proto::JobResponse>,
    pub machine_type: String,
}

#[derive(Debug)]
pub struct Request {
    pub tx: oneshot::Sender<Connection>,
    pub machine_type: String,
}

pub async fn task(
    mut request_rx: mpsc::UnboundedReceiver<Request>,
    mut connection_rx: mpsc::UnboundedReceiver<Connection>,
) {
    // The order of elements is important! This vec forms a FIFO and requests from the beginning are favored over later events.
    // This ensures that we complete benches from users that submitted first without getting distracted with benches that were submitted later on
    let mut waiting_requests: Vec<Request> = vec![];
    let mut waiting_connections: Vec<Connection> = vec![];

    loop {
        tokio::select!(
            request = request_rx.recv() => if let Some(request) = request {
                waiting_requests.push(request);
            } else {
                return
            },
            connection = connection_rx.recv() => if let Some(connection) = connection {
                waiting_connections.push(connection);
            } else {
                return
            }
        );

        if let Some((connection_i, request_i)) = find_match(&waiting_connections, &waiting_requests)
        {
            let connection = waiting_connections.remove(connection_i);
            let request = waiting_requests.remove(request_i);
            request.tx.send(connection).unwrap();
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
