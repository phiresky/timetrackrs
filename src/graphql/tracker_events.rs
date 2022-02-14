use crate::prelude::*;
use graphql_client::{GraphQLQuery, Response};
use anyhow::anyhow;

use super::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/insertTrackerEvents.graphql",
    response_derives = "Debug",
    normalization = "rust"
)]
pub struct InsertEvent;

impl From<NewDbEvent> for insert_event::Variables {
    fn from(new_db_event: NewDbEvent) -> Self {
        let NewDbEvent {
            timestamp_unix_ms,
            duration_ms,
            data_type,
            data,
            ..
        } = new_db_event;

        Self {
            timestamp: timestamp_unix_ms.0.timestamp(),
            duration: duration_ms,
            user_agent: data_type,
            data
        }
    }
}

pub async fn insert_tracker_event(new_db_event: NewDbEvent) -> anyhow::Result<i64> {
    use insert_event::{Variables, ResponseData};
    
    let variables: Variables = new_db_event.into();

    let query = InsertEvent::build_query(variables);

    let res = CLIENT
        .post("https://apps.crewnew.com/v1/graphql")
        .json(&query)
        .send()
        .await?;
    
    log::debug!("{:?}", res);

    let res_body: Response<ResponseData> = res.json().await?;
    
    log::debug!("{:?}", res_body);

    if let Some(err) = res_body.errors {
        anyhow::bail!("{:?}", err);
    }

    let affected_rows = res_body
        .data
        .ok_or_else(|| anyhow!("ResponseData is None"))?
        .insert_event
        .ok_or_else(|| anyhow!("InsertEventsInsertEventRaw is None"))?
        .affected_rows;
    log::debug!("{}", affected_rows);
    
    Ok(affected_rows)
}
