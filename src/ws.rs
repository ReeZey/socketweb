use std::path::PathBuf;
use actix_web::{web, HttpRequest, Responder};
use actix_ws::Message;
use futures_util::StreamExt;
use serde_json::Value;
use tokio::io::AsyncWriteExt;

pub async fn connection(req: HttpRequest, body: web::Payload) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                Message::Text(msg) => {
                    let request = serde_json::from_str::<Value>(&msg).unwrap();

                    match request.get("type").unwrap_or(&Value::Null) {
                        Value::String(str) => {
                            match str.as_str() {
                                "lookup" => {
                                    let mut file_name = request["path"].as_str().unwrap().to_string();

                                    if !file_name.starts_with("/") {
                                        file_name.insert(0, '/');
                                    }

                                    if file_name.ends_with("/") {
                                        file_name.push_str("index.html");
                                    }
                                    
                                    let path = PathBuf::from(format!("{}{}", crate::BASE_HTML, file_name));
                                    println!("Looking up file: {:?}", path.display());
                                    
                                    let (json_header, file_data) = match path.exists() {
                                        true => {
                                            println!("File found: {}", path.display());
                                            let file_read = tokio::fs::read(&path).await.unwrap();

                                            let mime_type = mimetype::detect(&file_read);

                                            println!("Detected MIME type: {}", mime_type.mime);

                                            let mut json = serde_json::json!({
                                                "type": "lookup",
                                                "path": path,
                                                "mime": mime_type.mime,
                                            });

                                            if let Some(elem) = request.get("elem") {
                                                json["elem"] = elem.clone();
                                            }

                                            (json.to_string(), Some(file_read))
                                        },
                                        false => (serde_json::json!({
                                            "type": "lookup",
                                            "error": "File not found",
                                        }).to_string(), None),
                                    };
                                    
                                    let header = json_header.as_bytes();
                                
                                    let mut response: Vec<u8> = vec![];
                                    response.write_u32(header.len() as u32).await.unwrap();
                                    response.write_all(header).await.unwrap();

                                    if let Some(file_read) = file_data {
                                        response.write_u32(file_read.len() as u32).await.unwrap();
                                        response.write_all(&file_read).await.unwrap();
                                    } 

                                    if session.binary(response).await.is_err() {
                                        return;
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                },
                _ => break,
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}