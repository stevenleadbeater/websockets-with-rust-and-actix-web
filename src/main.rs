use actix_cors::Cors;
use actix_web::{web, Responder, HttpResponse, HttpServer, App, get, HttpRequest};
use actix::{Actor, StreamHandler, Handler};
use actix_web_actors::ws;
use std::fmt::Debug;
use serde::{Serialize};
use actix::prelude::*;
use tokio::task;
use std::thread::sleep;
use std::time::Duration;


#[get("/")]
async fn get() -> impl Responder {
    println!("GET /");
    HttpResponse::Ok().body("test")
}

/// Define http actor
struct MyWs;

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Payload<T> {
    pub payload: T,
}

impl<T> Handler<Payload<T>> for MyWs where T: Serialize + Debug {
    type Result = ();

    fn handle(&mut self, msg: Payload<T>, ctx: &mut Self::Context) {
        println!("handle {:?}", msg.payload);
        ctx.text(serde_json::to_string(&msg.payload).expect("Cannot serialize"));
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn index(req: HttpRequest, stream: web::Payload) -> HttpResponse {
    let (addr, resp) = ws::start_with_addr(MyWs {}, &req, stream).unwrap();
    let recipient = addr.recipient();
    task::spawn(async move {
        loop {
            println!("Send ping");
            let result = recipient.send(Payload { payload: "ping".to_string() });
            let result = result.await;
            result.unwrap();
            sleep(Duration::from_secs(1));
        }
    });
    println!("{:?}", resp);
    resp
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("Started");
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::new().send_wildcard().finish())
            .service(get)
            .route("/ws/", web::get().to(index))
    })
        .bind("0.0.0.0:8120")?
        .run()
        .await
}