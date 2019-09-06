use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use futures::future::{ok, FutureResult};
use futures::{Future, Poll};
use crate::db::DbReplica;
use super::Auth;

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
    type Future = Box<dyn Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        Auth::from_request(&req);

        Box::new(self.service.call(req).map(move |res| {
            res
        }))
    }
}
