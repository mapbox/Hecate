use actix_service::{Service, Transform};
use actix_web::{http, dev::ServiceRequest, dev::ServiceResponse, Error, HttpResponse};
use futures::future::{ok, FutureResult, Either};
use futures::Poll;
use crate::db::DbReplica;
use super::{Auth, AuthDefault};

#[derive(Clone)]
pub struct EnforceAuth {
    db: DbReplica,
    auth: super::AuthDefault
}

impl EnforceAuth {
    pub fn new(db: DbReplica, auth: super::AuthDefault) -> EnforceAuth {
        EnforceAuth {
            db: db,
            auth: auth
        }
    }
}

impl<S, B> Transform<S> for EnforceAuth
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = EnforceAuthMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(EnforceAuthMiddleware {
            service,
            db: self.db.clone(),
            auth: self.auth.clone()
        })
    }
}

pub struct EnforceAuthMiddleware<S> {
    service: S,
    db: DbReplica,
    auth: super::AuthDefault
}

impl<S, B> Service for EnforceAuthMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, FutureResult<Self::Response, Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        let db = match self.db.get() {
            Ok(db) => db,
            Err(_err) => {
                return Either::B(ok(req.into_response(
                    HttpResponse::InternalServerError()
                        .finish()
                        .into_body(),
                )));
            }
        };

        let auth = match Auth::from_sreq(&mut req, &*db) {
            Err(_err) => {
                return Either::B(ok(req.into_response(
                    HttpResponse::Unauthorized()
                        .finish()
                        .into_body()
                )));
            },
            Ok(auth) => auth
        };

        // If no default auth is set - allow all api endpoints
        if self.auth == AuthDefault::Public {
            auth.as_headers(&mut req);
            return Either::A(self.service.call(req));
        }

        //Session Management is always allowed
        if req.path() == "/api/user/session" || req.path() == "/" {
            auth.as_headers(&mut req);
            return Either::A(self.service.call(req))
        }

        if
            auth.uid.is_none()
            || self.auth == AuthDefault::Admin && auth.access == Some(String::from("admin"))
        {
            let path: Vec<String> = req.path().split("/").map(|p| {
                p.to_string()
            }).filter(|p| {
                if p.len() == 0 {
                    return false;
                }

                return true;
            }).collect();

            if path.len() >= 1 && path[0] == String::from("admin") {
                // UI Results should redirect to an unauthenticated login portal
                // or allowed if they are for the login page

                if path.len() >= 2 && path[1] == String::from("login") {
                    return Either::A(self.service.call(req));
                } else {
                    return Either::B(ok(req.into_response(
                        HttpResponse::Found()
                            .header(http::header::LOCATION, "/admin/login/index.html")
                            .finish()
                            .into_body(),
                    )));
                }
            } else {
                // API Results should simply return a 401
                return Either::B(ok(req.into_response(
                    HttpResponse::Unauthorized()
                        .finish()
                        .into_body(),
                )));
            }
        }

        auth.as_headers(&mut req);
        Either::A(self.service.call(req))
    }
}
