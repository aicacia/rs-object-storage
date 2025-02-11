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
pub struct Pagination {
    #[serde(rename = "has_more")]
    pub has_more: bool,
    #[serde(rename = "items")]
    pub items: Vec<models::ObjectInstance>,
}

impl Pagination {
    pub fn new(has_more: bool, items: Vec<models::ObjectInstance>) -> Pagination {
        Pagination {
            has_more,
            items,
        }
    }
}

