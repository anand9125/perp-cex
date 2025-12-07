use db::Db;
use tokio::sync::mpsc::Sender;

use crate::types::OrderBookMessage;

pub struct AppState{
    pub book_tx : Sender<OrderBookMessage>,
    pub db: Db
}