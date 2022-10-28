use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use serde_derive::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::ops::Deref;
use std::sync::Mutex;
use uuid::Uuid;

#[macro_use]
extern crate lazy_static;

#[derive(Serialize, Deserialize)]
struct Poll {
    id: String,
    title: String,
    description: String,
    x: Vec<String>,
    y: Option<Vec<String>>,
    responses: Vec<PollResponse>,
}

#[derive(Serialize, Deserialize)]
struct PollResponse {
    name: String,
    selections: Vec<Selection>,
}

#[derive(Serialize, Deserialize)]
struct Selection {
    x: String,
    y: Option<String>,
    selection: String,
}

#[derive(Serialize, Deserialize)]
struct CreatePollRequest {
    title: String,
    description: String,
    x: Vec<String>,
    y: Option<Vec<String>>,
}

lazy_static! {
    static ref POLLS: Mutex<Vec<Poll>> = Mutex::new(Vec::new());
}

#[get("/poll")]
async fn get_polls() -> impl Responder {
    match serde_json::to_string(&POLLS.lock().unwrap().deref()) {
        Ok(ps) => HttpResponse::Ok().body(ps),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/poll")]
async fn create_poll(req_body: String) -> impl Responder {
    let r: Result<CreatePollRequest, serde_json::Error> = serde_json::from_str(&req_body);
    match r {
        Ok(req) => {
            let id = Uuid::new_v4().to_string();
            let poll: Poll = Poll {
                id: id.clone(),
                title: req.title,
                description: req.description,
                x: req.x,
                y: req.y,
                responses: Vec::new(),
            };
            POLLS.lock().unwrap().push(poll);
            HttpResponse::Ok().body(id)
        }
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[get("/poll/{id}")]
async fn get_poll(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    let polls = POLLS.lock().unwrap();
    let poll: Option<&Poll> = polls.iter().find(|&p| p.id == id);

    match poll {
        Some(p) => match serde_json::to_string(p) {
            Ok(p2) => HttpResponse::Ok().body(p2),
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        },
        None => HttpResponse::NotFound().body(format!("No poll with id {id}")),
    }
}

#[post("/poll/respond/{id}")]
async fn respond_to_poll(path: web::Path<String>, req_body: String) -> impl Responder {
    let id = path.into_inner();
    let mut polls = POLLS.lock().unwrap();
    let poll_index: Option<usize> = polls.iter().position(|p| p.id == id);

    match serde_json::from_str(&req_body) {
        Ok(response) => match poll_index {
            Some(index) => {
                polls[index].responses.push(response);
                HttpResponse::Ok().finish()
            }
            None => HttpResponse::NotFound().body(format!("No poll with id {id}")),
        },
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let port_string = env::var("PORT").unwrap_or("3001".to_string());
    let port = port_string.parse::<u16>().expect("PORT must be a number");
    println!("Starting on port {port}");

    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .service(create_poll)
            .service(get_poll)
            .service(respond_to_poll)
            .service(get_polls)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
