use actix_web::{App, HttpServer, web};
use db::Db;
pub mod auth;
pub use auth::*;
pub mod models;
pub use models::*;

#[actix_web::main]
async fn main(){
    dotenvy::dotenv().ok();
    let db = Db::new().await.expect("failed toconnecte to database");
    let _ = HttpServer::new(move || {
        App::new()
            .service(web::resource("/signup").route(web::post().to(create_user)))
            .service(web::resource("/signin").route(web::post().to(signin)))
            .app_data(actix_web::web::Data::new(db.clone()))
            .service(web::scope("/api")
                            .wrap(JwtMiddleware)
                            .route("/me", web::get().to(me_handler))
     
            )
    })
    .bind("0.0.0.0:3000")
    .unwrap()
    .run()
    .await;
}