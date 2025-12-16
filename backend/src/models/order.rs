use actix_web::{ Responder, Result, http::StatusCode, web::{self, Json}};
use rust_decimal::{Decimal, prelude::{FromPrimitive}};
use rust_decimal_macros::dec;
use tokio::sync::oneshot;

use crate::{LimitOrder, MarketOrder, Order, OrderResponse, state::AppState, types::{ CanceledOrderRequest, OrderBookMessage, OrderRequest, OrderType, Response}};


pub async fn place_order(
    body: Json<OrderRequest>,
    state:web::Data<AppState>
)->impl Responder{
    let req = body.into_inner();
    let (tx, rx) = oneshot::channel::<Result<OrderResponse,String>>();

    let quantity = match Decimal::from_f64(req.quantity){
        Some(q) if q > dec!(0) =>q,
        _ => {
            return (
                Json(Response{
                    message: String::new(),
                    error: "Invalid quantity".to_string(),
                }),
                StatusCode::BAD_REQUEST
            );
        }
    };

    let leverage = Decimal::from_u32(req.leverage).unwrap_or(dec!(1));
    let order = match req.type_{
        OrderType::Limit =>{
            let price_f64 = match req.price{
                Some(p)=>p,
                None=> {
                    return (
                        Json(Response{
                            message : String::new(),
                            error:"you should have to give the price".to_string()
                        }),
                        StatusCode::BAD_REQUEST
                    );
                }
            };
            let price = match Decimal::from_f64(price_f64){
                Some(p)if p >dec!(0)=>p,
                _ =>{
                    return (
                        Json(
                            Response{
                                message : String::new(),
                                error : "error while converting ".to_string()
                            }
                        ),
                        StatusCode::BAD_REQUEST
                    );
                }
            };
            Order::limit_order(LimitOrder {
                user_id: req.user_id,
                side: req.side,
                price,
                quantity,
                leverage,
            })
        }
        OrderType::Market =>{
             if req.price.is_some() {
                return (
                    Json(Response {
                        message: String::new(),
                        error: "Market order must not include price".to_string(),
                    }),
                    StatusCode::BAD_REQUEST,
                );
            }
            Order::market_order(MarketOrder {
                user_id: req.user_id,
                side: req.side,
                quantity,
                leverage,
            })
        }
    };
    if let Err(_) = state.book_tx.send(OrderBookMessage::PlaceOrder { 
        order,
        priority: crate::types::Priority::Normal,
        responder: Some(tx)
    }){
        return (
            Json(Response{
                message:String::new(),
                error : "Engine unavailable".to_string()
            }),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }
    match rx.await {
        Ok(Ok(OrderResponse::PlacedOrder { 
            order_id,
            status, 
            filled,
            remaining
        })) => (
            Json(Response {
                message: format!(
                    "order processed: filled {},status,{} remaining {}, {}",
                    filled,status, remaining,order_id
                ),
                error: String::new(),
            }),
            StatusCode::OK,
        ),

        _ => (
            Json(Response {
                message: String::new(),
                error: "Engine response dropped".to_string(),
            }),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
    
}


pub async fn cancel_order(
    body: Json<CanceledOrderRequest>,
    state : web::Data<AppState>
)-> impl Responder{
    let req = body.into_inner();
    let (tx, rx) = oneshot::channel::<Result<OrderResponse,String>>();

    let order_id = req.order_id;
    let user_id = req.user_id;

    if let Err(_) = state.book_tx.send(OrderBookMessage::CancelOrder { 
        user_id,
        order_id,
        responder:Some(tx)
     }){
        return (
            Json(Response{
                message : String::new(),
                error : "Erro while sending throug mpsc".to_string()
            }),
            StatusCode::BAD_REQUEST 
        );
    };

    match rx.await{
        Ok(Ok(OrderResponse::CanceledOrder {
            order_id,
            user_id,
            status,
            message 
        }))=>(
                Json(Response{
                    message : format!(
                        "Order cancelled successfull : order_id {}, user_id {},status {},message{}",
                        order_id,user_id,status,message
                    ),
                    error : String::new()
                }),
                StatusCode::OK
        ),
        _ => (
            Json(Response{
                message : String::new(),
                error : "eningne respone".to_string()
            }),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
    
}