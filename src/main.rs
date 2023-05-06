use simple_websockets::{Event, Responder};
use std::collections::HashMap;
use sqlx::{SqlitePool, Error};
use tokio::runtime::Runtime;

fn main() {
    let event_hub = simple_websockets::launch(8080).expect("failed to listen on port 8080");
    let mut clients: HashMap<u64, (Option<String>, Responder)> = HashMap::new();
    let mut messages: HashMap<String, Vec<String>> = HashMap::new();

    let rt = Runtime::new().expect("Failed to create a runtime");
    let conn = rt.block_on(initialize_database()).expect("Failed to initialize the database");

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
                        rt.block_on(store_username(&conn, client_id.try_into().unwrap(), &msg_str))
                            .expect("Failed to store username in the database");
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

                        rt.block_on(store_message(&conn, user, &msg_str))
                            .expect("Failed to store message in the database");

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

async fn initialize_database() -> Result<SqlitePool, Error> {
    let pool = SqlitePool::connect("sqlite:chat.db").await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL UNIQUE
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            content TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users (id)
        )",
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

async fn store_username(pool: &SqlitePool, user_id: i64, username: &str) -> Result<(), Error> {
    sqlx::query("INSERT OR IGNORE INTO users (id, username) VALUES (?1, ?2)")
        .bind(user_id)
        .bind(username)
        .execute(pool)
        .await?;

    Ok(())
}

async fn store_message(pool: &SqlitePool, username: &str, content: &str) -> Result<(), Error> {
    let user_id: Option<i64> = sqlx::query_scalar("SELECT id FROM users WHERE username = ?1")
        .bind(username)
        .fetch_optional(pool)
        .await?;

    if let Some(user_id) = user_id {
        sqlx::query("INSERT INTO messages (user_id, content) VALUES (?1, ?2)")
            .bind(user_id)
            .bind(content)
            .execute(pool)
            .await?;
    } else {
        eprintln!("Failed to find user with username: {}", username);
    }

    Ok(())
}