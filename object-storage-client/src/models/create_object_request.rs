/*
 * object-storage
 *
 * Aicacia Object Storage API provides blob services for applications.
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateObjectRequest {
    #[serde(rename = "path")]
    pub path: String,
    #[serde(rename = "type", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Option<String>>,
}

impl CreateObjectRequest {
    pub fn new(path: String) -> CreateObjectRequest {
        CreateObjectRequest {
            path,
            r#type: None,
        }
    }
}

