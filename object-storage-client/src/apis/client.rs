use std::sync::Arc;

use hyper;
use hyper_util::client::legacy::connect::Connect;
use super::configuration::Configuration;

pub struct APIClient {
    object_api: Box<dyn crate::apis::ObjectApi>,
    openapi_api: Box<dyn crate::apis::OpenapiApi>,
    p2p_api: Box<dyn crate::apis::P2pApi>,
    util_api: Box<dyn crate::apis::UtilApi>,
}

impl APIClient {
    pub fn new<C: Connect>(configuration: Configuration<C>) -> APIClient
        where C: Clone + std::marker::Send + Sync + 'static {
        let rc = Arc::new(configuration);

        APIClient {
            object_api: Box::new(crate::apis::ObjectApiClient::new(rc.clone())),
            openapi_api: Box::new(crate::apis::OpenapiApiClient::new(rc.clone())),
            p2p_api: Box::new(crate::apis::P2pApiClient::new(rc.clone())),
            util_api: Box::new(crate::apis::UtilApiClient::new(rc.clone())),
        }
    }

    pub fn object_api(&self) -> &dyn crate::apis::ObjectApi{
        self.object_api.as_ref()
    }

    pub fn openapi_api(&self) -> &dyn crate::apis::OpenapiApi{
        self.openapi_api.as_ref()
    }

    pub fn p2p_api(&self) -> &dyn crate::apis::P2pApi{
        self.p2p_api.as_ref()
    }

    pub fn util_api(&self) -> &dyn crate::apis::UtilApi{
        self.util_api.as_ref()
    }

}
