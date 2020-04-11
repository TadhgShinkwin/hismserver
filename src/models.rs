use crate::errors::AppError;
use crate::schema::{users};
use diesel::prelude::*;

//just return Result<T> and not have to written AppError everywhere.
type Result<T> = std::result::Result<T, AppError>;


//Queryable: Diesel represents the return type of a query as a tuple. The purpose of this
// trait is to convert from a tuple of Rust values that have been deserialized into your struct.

//Identifiable: is a trait that indicates that this struct represents a single row in a
// database table. It assumes a primary key named id but you can configure this.
#[derive(Queryable, Identifiable, Serialize, Debug, PartialEq)]
//We need a Rust struct to represent a user in the database
pub struct User {
    pub id: i32,
    pub username: String,
}

pub fn create_user(conn: &SqliteConnection, username: &str) -> Result<User> {
    //The connection type supports a method transaction which takes a closure. The closure must return a Result.
    conn.transaction(|| {
        diesel::insert_into(users::table)
            .values((users::username.eq(username),))
            // error Result with our AppError error type because of our From implementation in our errors module.
            .execute(conn)?;

        //Sqlite does not support getting the id of a just inserted row as part of the insert statement.
        // to actually get the data back out to build a User struct we do another query.
        users::table
            .order(users::id.desc())
            .select((users::id, users::username))
            .first(conn)
            //uses the function signature to determine what to transform the error into.
            .map_err(Into::into)
    })
}

pub enum UserKey<'a> {
    Username(&'a str),
    ID(i32),
}

pub fn find_user<'a>(conn: &SqliteConnection,key: UserKey<'a>) -> Result<User> {
    match key {
        UserKey::Username(name) => users::table
            .filter(users::username.eq(name))
            .select((users::id, users::username))
            .first::<User>(conn)
            .map_err(AppError::from),
        UserKey::ID(id) => users::table
            .find(id)
            .select((users::id, users::username))
            .first::<User>(conn)
            .map_err(Into::into),
    }
}