#![allow(dead_code)]
use crate::setup::WggClient;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Map;

#[derive(Debug, Deserialize)]
pub struct GraphQLCustomResponse {
    pub data: serde_json::Value,
    pub errors: Option<serde_json::Value>,
}

impl GraphQLCustomResponse {
    pub fn to_pagination(&self, root: &str) -> PaginatedResponse {
        let root = &self.data[root];

        PaginatedResponse {
            total_count: root["totalCount"].as_u64(),
            page_info: serde_json::from_value(root["pageInfo"].clone()).unwrap(),
            items: root["edges"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .map(|b| b["node"].clone())
                .collect::<Vec<serde_json::Value>>(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct GraphQLCustomRequest {
    pub query: String,
    pub variables: Map<String, serde_json::Value>,
}

impl GraphQLCustomRequest {
    pub fn from_query(query: impl Into<String>) -> Self {
        GraphQLCustomRequest {
            query: query.into(),
            variables: Map::new(),
        }
    }

    pub fn with_variable<T: Serialize, S: Into<String>>(mut self, key: S, value: T) -> Self {
        self.variables.insert(key.into(), serde_json::to_value(value).unwrap());

        self
    }
}

impl From<&str> for GraphQLCustomRequest {
    fn from(input: &str) -> Self {
        GraphQLCustomRequest::from_query(input)
    }
}

impl From<(&str, Map<String, serde_json::Value>)> for GraphQLCustomRequest {
    fn from(input: (&str, Map<String, serde_json::Value>)) -> Self {
        GraphQLCustomRequest {
            query: input.0.to_string(),
            variables: input.1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse {
    pub total_count: Option<u64>,
    pub page_info: Option<PageInfo>,
    pub items: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageInfo {
    pub has_previous_page: Option<bool>,
    pub has_next_page: Option<bool>,
    pub start_cursor: Option<u64>,
    pub end_cursor: Option<u64>,
}

pub async fn post_graphql_request(
    client: &WggClient,
    request: GraphQLCustomRequest,
) -> anyhow::Result<GraphQLCustomResponse> {
    let response = client
        .post("/api/graphql")
        .json(&request)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to query Graphql API Route: \n{:#?}", e))?;

    let result: GraphQLCustomResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Graphql had an error, failed to parse to json: {:#?}", e))?;

    if result.errors.is_some() {
        return Err(anyhow!("Graphql had an error: {:#?}", result.errors));
    }

    Ok(result)
}
