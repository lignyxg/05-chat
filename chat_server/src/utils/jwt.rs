use std::collections::HashSet;

use jwt_simple::algorithms::{ECDSAP256KeyPairLike, ECDSAP256PublicKeyLike, ES256KeyPair};
use jwt_simple::claims::Claims;
use jwt_simple::common::VerificationOptions;
use jwt_simple::prelude::Duration;

use crate::error::AppError;
use crate::models::User;

const JWT_DURATION: u64 = 7 * 24 * 60 * 60;
const JWT_ISSUER: &str = "chat_server";
const JWT_AUDIENCE: &str = "chat_client";
pub struct JwtSigner {
    pub(crate) kpair: ES256KeyPair,
}

#[allow(unused)]
impl JwtSigner {
    pub(crate) fn new(kpair: ES256KeyPair) -> Self {
        Self { kpair }
    }

    pub(crate) fn load(path: &str) -> Result<Self, AppError> {
        let pem = std::fs::read_to_string(path)?;
        let kpair = ES256KeyPair::from_pem(&pem)?;
        Ok(Self::new(kpair))
    }

    pub(crate) fn sign(&self, user: impl Into<User>) -> Result<String, AppError> {
        let claims = Claims::with_custom_claims(user.into(), Duration::from_secs(JWT_DURATION))
            .with_issuer(JWT_ISSUER)
            .with_audience(JWT_AUDIENCE);
        let token = self.kpair.sign(claims)?;

        Ok(token)
    }

    pub(crate) fn verify(&self, token: &str) -> Result<User, AppError> {
        let allowed_issuers = HashSet::from([JWT_ISSUER.to_string()]);
        let allowed_audiences = HashSet::from([JWT_AUDIENCE.to_string()]);
        let opts = VerificationOptions {
            allowed_issuers: Some(allowed_issuers),
            allowed_audiences: Some(allowed_audiences),
            max_validity: Some(Duration::from_secs(JWT_DURATION)),
            ..Default::default()
        };
        let pub_key = self.kpair.public_key();
        let claims = pub_key.verify_token::<User>(token, Some(opts))?;

        Ok(claims.custom)
    }
}
