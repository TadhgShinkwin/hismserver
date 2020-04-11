use crate::errors::AppError;
use crate::routes::convert;
use crate::{models, Pool};
use actix_web::{web, HttpResponse};
use futures::Future;

// mapping URLs to routes which have methods and handles
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/users").route(web::post().to_async(create_user)))
        .service(web::resource("/users/find/{name}").route(web::get().to_async(find_user)))
        .service(web::resource("/users/{id}").route(web::get().to_async(get_user)));
}


#[derive(Debug, Serialize, Deserialize)]
struct UserInput {
    username: String,
}

//these functions get a connection from the pool (in lib.rs) and call models to communicate with the database to insert or get data

fn create_user(
    item: web::Json<UserInput>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AppError> {
    //executes a blocking function on a thread pool and returns a future that resolves to the result of the function execution.
    web::block(move || {
    let conn = &pool.get().unwrap();
    let username = item.into_inner().username;
    models::create_user(conn, username.as_str())
    })
        //uses convert in routes to turn result into HttpResponse or AppError from errors
    .then(convert)
}

fn find_user(
    name: web::Path<String>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AppError> {
    web::block(move || {
        let conn = &pool.get().unwrap();
        let name = name.into_inner();
        let key = models::UserKey::Username(name.as_str());
        models::find_user(conn, key)
    })
        .then(convert)
}

fn get_user(
    user_id: web::Path<i32>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error= AppError> {
    web::block(move || {
        let conn = &pool.get().unwrap();
        let id = user_id.into_inner();
        let key = models::UserKey::ID(id);
        models::find_user(conn, key)
    })
        .then(convert)
}