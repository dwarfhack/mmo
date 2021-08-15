use actix_web::{get, App, HttpServer, Responder};

#[get("/")]
async fn index() -> impl Responder {
    format!("Hello")
}

#[actix_web::main]
async fn not_main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

#[cfg(test)]
mod tests {

    use std::{thread::sleep, time::Duration};


    use crate::not_main;

    #[test]
    fn test_api_starts_without_crash() {
        
        let _ = std::thread::spawn(||{
            let _ = not_main();
        });

        sleep(Duration::from_secs(1));
        
        //ok

    }
}
