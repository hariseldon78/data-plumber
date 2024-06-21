use crate::state::{read_config_field, Factory, Process, Record, State, Table, Variant};
use crate::register_process;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::result::Result;
// use futures_lite::stream::StreamExt;
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Connection,
    ConnectionProperties,
};
use tracing::info;
use async_global_executor;

pub struct OutputRabbitMQ {
    pub input: String,
    pub exchange: String,
    pub url: String,
    pub body: String,
}

impl Process for OutputRabbitMQ {
    register_process!(output::rabbitmq);
    fn from_config(node_name: String, config: Map<String, Value>) -> Self {
        OutputRabbitMQ {
            input: read_config_field(&config, "input"),
            exchange: read_config_field(&config, "exchange"),
            url: read_config_field(&config, "url"),
            body: read_config_field(&config, "body"),
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

            let queue = channel_a
                .queue_declare(
                    self.exchange.as_str(),
                    QueueDeclareOptions::default(),
                    FieldTable::default(),
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
