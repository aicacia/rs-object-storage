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
pub struct ObjectInstance {
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "id")]
    pub id: i64,
    #[serde(rename = "path")]
    pub path: String,
    #[serde(rename = "size")]
    pub size: u64,
    #[serde(rename = "type", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Option<String>>,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
}

impl ObjectInstance {
    pub fn new(created_at: String, id: i64, path: String, size: u64, updated_at: String) -> ObjectInstance {
        ObjectInstance {
            created_at,
            id,
            path,
            size,
            r#type: None,
            updated_at,
        }
    }
}

