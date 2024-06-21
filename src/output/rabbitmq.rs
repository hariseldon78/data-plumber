use crate::register_process;
use crate::state::{read_config_field, Factory, Process, Record, State, Table, Variant};
use lapin::ExchangeKind;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::result::Result;
// use futures_lite::stream::StreamExt;
use async_global_executor;
use lapin::{
    options::*, publisher_confirm::Confirmation, types::AMQPType, types::AMQPValue,
    types::FieldTable, BasicProperties, Connection, ConnectionProperties,
};
use tracing::info;

pub struct OutputRabbitMQ {
    pub input: String,
    pub exchange: String,
    pub routing_key: String,
    pub url: String,
    pub body: String,
    pub exchange_options: HashMap<String, Value>,
    pub dry_run: bool,
}

impl Process for OutputRabbitMQ {
    register_process!(output::rabbitmq);
    fn from_config(node_name: String, config: Map<String, Value>) -> Self {
        let exchange_options = config
            .get("exchange_options")
            .unwrap_or(&Value::Null)
            .as_object()
            .unwrap_or(&Map::new())
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        OutputRabbitMQ {
            input: read_config_field(&config, "input"),
            exchange: read_config_field(&config, "exchange"),
            routing_key: read_config_field(&config, "routing_key"),
            url: read_config_field(&config, "url"),
            body: read_config_field(&config, "body"),
            exchange_options,
            dry_run: config
                .get("dry_run")
                .unwrap_or(&Value::Bool(false))
                .as_bool()
                .unwrap(),
        }
    }
    fn run(&self, state: &mut State) {
        let table = state.find_table(&self.input).unwrap();

        async_global_executor::block_on(async {
            let conn = Connection::connect(&(self.url), ConnectionProperties::default())
                .await
                .unwrap();

            let channel_a = conn.create_channel().await.unwrap();

            let mut exchange_args = FieldTable::default();

            for (key, value) in &self.exchange_options {
                let amqp_value = match value {
                    Value::String(s) => AMQPValue::try_from(value, AMQPType::LongString).unwrap(),
                    Value::Number(n) => AMQPValue::LongLongInt(n.as_i64().unwrap()),
                    Value::Bool(b) => AMQPValue::Boolean(b.clone()),
                    _ => AMQPValue::Void,
                };
                exchange_args.insert(key.clone().into(), amqp_value);
            }

            let exchange = channel_a
                .queue_declare(
                    &self.exchange.as_str(),
                    QueueDeclareOptions::default(),
                    exchange_args,
                )
                .await
                .unwrap();

            for record in &(table.records) {
                let mut payload = self.body.clone();
                for (key, value) in &record.fields {
                    let key = key.clone();
                    let value = value.to_string();
                    payload = payload.replace(&format!("{{{{{}}}}}", key), &value.to_string());
                }
                if self.dry_run {
                    println!("[would be] sending message to rabbitmq: {}", payload);
                } else {
                    println!("sending message to rabbitmq: {}", payload);
                    let confirm = channel_a
                        .basic_publish(
                            self.exchange.as_str(),
                            self.routing_key.as_str(),
                            BasicPublishOptions::default(),
                            payload.as_bytes(),
                            BasicProperties::default(),
                        )
                        .await
                        .unwrap();
                    confirm.await.unwrap();
                }
            }
        });
    }
}
