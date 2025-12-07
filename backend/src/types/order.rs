use rust_decimal::Decimal;
pub use serde::{Serialize,Deserialize};
use tokio::sync::oneshot;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct OrderRequest {
    #[serde(rename = "type")]
    pub type_: OrderType,
    pub user_id : Uuid,
    pub side: Side,
    pub amount: f64,
    pub price: f64,
    pub leverage: u32,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub message: String,
    pub error: String,
}

pub struct Order {
    pub order_id : Uuid,
    pub user_id : Uuid,
    pub price : Decimal,
    pub amount : Decimal,
    pub leverage : Decimal,
    pub side : Side,
    pub responder : Option<oneshot::Sender<OrderResponse>>  
}

#[derive(Clone)]
pub struct OrderResponse{
    pub status : String,
    pub filled : Decimal,
    pub remaining : Decimal
}


pub enum OrderBookMessage{
    Order(Order)  //Enum Variants With Data = Structs Inside an Enum
}