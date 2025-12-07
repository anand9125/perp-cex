use actix_web::{HttpRequest, HttpResponse, error::{ErrorConflict, ErrorUnauthorized}, web::{Data, Json}};
use db::{Db};
use actix_web::HttpMessage;

use crate::{create_jwt, types::{SinginResponse, UserRequest, UserResponse}};




pub async  fn create_user(db:Data<Db>,body:Json<UserRequest>)->Result<Json<UserResponse>,actix_web::error::Error>{
    let user = db.create_user(&body.email,&body.password)
        .await
        .map_err(|e|ErrorConflict(e.to_string()))?;

    Ok(Json(UserResponse { id: user.id}))
}



pub async fn signin(db:Data<Db>,body:Json<UserRequest>)->Result<Json<SinginResponse>,actix_web::error::Error>{
    let user = db.get_user(&body.email)
        .await
        .map_err(|e|ErrorConflict(e.to_string()))?;

    if user.password != body.password {
        return Err(ErrorUnauthorized("Invalid email or password"));
    }
    let token = create_jwt(user.id);
    Ok(Json(SinginResponse{
        token
    }))
}

pub async fn me_handler(req: HttpRequest) -> HttpResponse {
    let user_id = req.extensions().get::<i32>().cloned();
    HttpResponse::Ok().body(format!("User = {:?}", user_id))
}
