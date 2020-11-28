use warp::{http::StatusCode, reject, Reply, Rejection, Filter};
use serde::{Serialize, Deserialize};
use crate::models::models::{Account, Debtor, Item};
use std::convert::Infallible;
use crate::models::Db;
use serde_json::json;

extern crate rand;

use rand::Rng; 
use rand::distributions::Alphanumeric;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateAccount{
    pub items: Vec<Item>,
    pub debtors: Vec<Debtor>,
    pub name: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct CustomError{
    pub error: String
}

fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn json_body() -> impl Filter<Extract = (CreateAccount,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

pub fn events_end(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        (event_create(db.clone()))
}

/// POST /todos with JSON body
pub fn event_create(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("events")
        .and(warp::post())
        .and(json_body())
        .and(with_db(db))
        .and_then(create_event)
}



pub async fn event_info(id: String, db: Db) -> Result<impl warp::Reply, Infallible>{

    let accounts = db.lock().await;

    let account: Option<&Account> = accounts.iter().find(|a| a.id == id);

    match account{
        Some(acc) => Ok(            
            warp::reply::with_status(
                warp::reply::json(&acc),
                StatusCode::CREATED
            )
        ),
        None =>  Ok(
            warp::reply::with_status(
                warp::reply::json(&CustomError{error:"Evento no encontrado".to_string()}),
                StatusCode::NOT_FOUND
            )
        )
    }
}

pub async fn create_event(create: CreateAccount, db: Db) -> Result<impl warp::Reply, Infallible>{

    let id = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .collect::<String>(); 

    let acc: Account = Account{
        items: create.items,
        debtors: create.debtors,
        name: create.name,
        id
    };

    let mut accounts = db.lock().await;
    accounts.push(acc.clone());

    //.and(warp::reply::json(&acc)
    Ok(warp::reply::with_status(warp::reply::json(&acc), StatusCode::CREATED))
}