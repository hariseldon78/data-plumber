use crate::state::{read_config_field, Factory, Process, Record, State, Table, Variant};
use crate::register_process;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::result::Result;
// use futures_lite::stream::StreamExt;
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Connection,
    ConnectionProperties, types::AMQPValue, types::AMQPType,
};
use tracing::info;
use async_global_executor;

pub struct OutputRabbitMQ {
    pub input: String,
    pub exchange: String,
    pub url: String,
    pub body: String,
    pub queue_options: HashMap<String, Value>,
}

impl Process for OutputRabbitMQ {
    register_process!(output::rabbitmq);
    fn from_config(node_name: String, config: Map<String, Value>) -> Self {

        let queue_options= config
            .get("queue_options")
            .unwrap_or(&Value::Null)
            .as_object()
            .unwrap_or(&Map::new())
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();


        OutputRabbitMQ {
            input: read_config_field(&config, "input"),
            exchange: read_config_field(&config, "exchange"),
            url: read_config_field(&config, "url"),
            body: read_config_field(&config, "body"),
            queue_options,
        }
    }
    fn run(&self, state: &mut State) {
        let table = state.find_table(&self.input).unwrap();

    // let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());

        async_global_executor::block_on(async {
            let conn = Connection::connect(
                &(self.url),
                ConnectionProperties::default(),
            )
                .await.unwrap();

            info!("CONNECTED");

            let channel_a = conn.create_channel().await.unwrap();

            let mut queue_args = FieldTable::default();
            // queue_args.insert("x-message-ttl".into(), 1800000.into());
            // add all queue_options to queue_args
            for (key, value) in &self.queue_options {
                let amqp_value = match value {
                    Value::String(s) => AMQPValue::try_from(value,AMQPType::LongString).unwrap(),
                    Value::Number(n) => AMQPValue::LongLongInt(n.as_i64().unwrap()),
                    Value::Bool(b) => AMQPValue::Boolean(b.clone()),
                    _ => AMQPValue::Void,
                };
                queue_args.insert(key.clone().into(), amqp_value);
            }

            let queue = channel_a
                .queue_declare(
                    self.exchange.as_str(),
                    QueueDeclareOptions::default(),
                    queue_args,
                )
                .await.unwrap();

            for record in &(table.records) {
                let payload = self.body.clone();
                println!("{}", payload);
            }
        });
    }



        //     let confirm = channel_a
        //         .basic_publish(
        //             "",
        //             self.exchange.as_str(),
        //             BasicPublishOptions::default(),
        //             payload.as_bytes().to_vec(),
        //             BasicProperties::default(),
        //         )
        //         .await?
        //         .await?;
        //     assert_eq!(confirm, Confirmation::NotRequested);
        // }

        // loop {
        //     let confirm = channel_a
        //         .basic_publish(
        //             "",
        //             "hello",
        //             BasicPublishOptions::default(),
        //             payload,
        //             BasicProperties::default(),
        //         )
        //         .await?
        //         .await?;
        //     assert_eq!(confirm, Confirmation::NotRequested);
        // }
    // })
}
