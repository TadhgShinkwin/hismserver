use actix_web::error::BlockingError;
use actix_web::web::HttpResponse;
use diesel::result::DatabaseErrorKind::UniqueViolation;
use diesel::result::Error::{DatabaseError, NotFound};
use std::fmt;

//we want to return an error code and a message instead of continuing to process the
// request. This is a natural use case for an enumerated type.

/*Derive attribute on our struct  allows us to format instances of our type with the debug string formatter: {:?} .*/
#[derive(Debug)]
pub enum AppError {
    RecordAlreadyExists,
    RecordNotFound,
    DatabaseError(diesel::result::Error),
    OperationCanceled,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::RecordAlreadyExists => write!(f, "This record violates a unique constraint"),
            AppError::RecordNotFound => write!(f, "This record does not exist"),
            AppError::DatabaseError(e) => write!(f, "Database error: {:?}", e),
            AppError::OperationCanceled => write!(f, "The running operation was canceled"),
        }
    }
}

/*given an instance of diesel::result::Error and are expected to return an instance of AppError .*/
impl From<diesel::result::Error> for AppError {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            DatabaseError(UniqueViolation, _) => AppError::RecordAlreadyExists,
            NotFound => AppError::RecordNotFound,
            _ => AppError::DatabaseError(e),
        }
    }
}

//actix web specific error. Handlers will run blocking code which can either succeed
// or can fail because the future was canceled or the underlying blocking code returned an error.
impl From<BlockingError<AppError>> for AppError {
    fn from(e: BlockingError<AppError>) -> Self {
        match e {
            BlockingError::Error(inner) => inner,
            BlockingError::Canceled => AppError::OperationCanceled,
        }
    }
}

//JSON APIs should return JSON content or nothing as their error.
#[derive(Debug, Serialize)]
struct ErrorResponse {
    err: String,
}
//Actix web defines a trait ResponseError which allows you to specify how the type inside a Err variant of a Result gets turned into a response.
impl actix_web::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let err = format!("{}", self);
        let mut builder = match self {
            AppError::RecordAlreadyExists => HttpResponse::BadRequest(),
            AppError::RecordNotFound => HttpResponse::NotFound(),
            _ => HttpResponse::InternalServerError(),
        };
        builder.json(ErrorResponse { err })
    }
        //The trait also has a method render_response which has a default implementation, but the default overrides the content type and data which is not what we want.
        fn render_response(&self) -> HttpResponse {
            self.error_response()
        }

}