use simple_websockets::{Event, Responder};
use std::collections::HashMap;

fn main() {
    let event_hub = simple_websockets::launch(8080).expect("failed to listen on port 8080");
    let mut clients: HashMap<u64, (Option<String>, Responder)> = HashMap::new();
    let mut messages: HashMap<String, Vec<String>> = HashMap::new();

    loop {
        match event_hub.poll_event() {
            Event::Connect(client_id, responder) => {
                println!("A client connected with id #{}", client_id);
                clients.insert(client_id, (None, responder.clone()));
                responder.send(simple_websockets::Message::Text(
                    "Please choose a username:".to_string(),
                ));
            }
            Event::Disconnect(client_id) => {
                println!("Client #{} disconnected.", client_id);
                clients.remove(&client_id);
            }
            Event::Message(client_id, message) => {
                if let Some((username, _)) = clients.get(&client_id) {
                    let msg_str = match message {
                        simple_websockets::Message::Text(text) => text,
                        _ => continue,
                    };

                    if username.is_none() {
                        clients.get_mut(&client_id).unwrap().0 = Some(msg_str.clone());
                        clients.get_mut(&client_id).unwrap().1.send(
                            simple_websockets::Message::Text(format!(
                                "Username set to '{}'",
                                msg_str
                            )),
                        );
                    } else {
                        let user = username.as_ref().unwrap();
                        println!(
                            "Received a message from client #{} ({}): {:?}",
                            client_id, user, msg_str
                        );
                        messages
                            .entry(user.to_string())
                            .or_default()
                            .push(msg_str.clone());

                        // Broadcast message to all clients except the sender
                        for (other_client_id, (_, other_responder)) in &clients {
                            if *other_client_id != client_id {
                                // Construct the message inside the loop
                                let broadcast_msg = simple_websockets::Message::Text(format!(
                                    "{}: {}",
                                    user, msg_str
                                ));
                                other_responder.send(broadcast_msg);
                            }
                        }
                    }
                }
            }
        }
    }
}
