use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


use crate::Db;


#[derive(Serialize,Deserialize)]
pub struct  CreateUserResponse{
    pub id :Uuid
}

#[derive(Serialize,Deserialize)]
pub struct User {
    pub id : Uuid,
    pub email : String,
    pub password : String
}

impl Db {
    pub async fn create_user(&self,email:&String,password:&String)->Result<CreateUserResponse>{
        let u = sqlx::query_as!(CreateUserResponse,"INSERT INTO users (email,password) VALUES ($1,$2) RETURNING id",email,password)
           .fetch_one(&self.pool)
           .await?;
        Ok(CreateUserResponse { 
            id: u.id
        })

    }
    pub  async  fn get_user(&self , email:&String)->Result<User>{
        let u = sqlx::query_as!(User,"SELECT id , email,password FROM users WHERE email=$1",email)
            .fetch_one(&self.pool)
            .await?;
        Ok(u)
    }
}