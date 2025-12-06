use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, HttpMessage,
};
use actix_web::body::EitherBody;
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::{future::ready, rc::Rc};
use crate::Claims;

pub struct JwtMiddleware;

pub struct JwtMiddlewareService<S> {
    pub service: Rc<S>,
}

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = JwtMiddlewareService<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    B: 'static,
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {

            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok());

            if auth_header.is_none() || !auth_header.unwrap().starts_with("Bearer ") {
                let resp = HttpResponse::Unauthorized()
                    .body("Missing Token")
                    .map_into_right_body();
                return Ok(req.into_response(resp));
            }

          
            let token = auth_header.unwrap().trim_start_matches("Bearer ").trim();


            let secret = std::env::var("JWT_SECRET").unwrap();
            let validation = Validation::default();

            let decoded = decode::<Claims>(
                token,
                &DecodingKey::from_secret(secret.as_bytes()),
                &validation,
            );

            match decoded {
                Ok(data) => {
                    req.extensions_mut().insert(data.claims.sub);

                    let res = service.call(req).await?;
                    Ok(res.map_into_left_body())  
                }

                Err(_) => {
                    let resp = HttpResponse::Unauthorized()
                        .body("Invalid Token")
                        .map_into_right_body();
                    Ok(req.into_response(resp)) // error branch
                }
            }
        })
    }
}
