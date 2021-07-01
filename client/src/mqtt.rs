use crate::mqtt::Request::{Subscription, Publication};

use std::collections::HashMap;
use std::future::Future;
use std::sync::{
    Arc, Mutex, mpsc::{self, Sender, Receiver}
};

use rumqttc::{self, AsyncClient, Event, MqttOptions, Packet, QoS};
use tokio::task;
use tokio::task::JoinHandle;
use futures::future::{join3, Join3};
use delegate::delegate;
use Request::Unsubscription;
use log::{info, error};
use std::thread;
use futures::executor::block_on;
use tokio::runtime::Handle;
use regex::Regex;

pub enum Request {
    Subscription(String, Box<dyn Fn(&String, &String) + Send + 'static>),
    Unsubscription(String),
    Publication(String, String, bool),
}

pub struct Mosquitto {
    running: Arc<Mutex<bool>>,
    sender: Sender<Request>,
}

pub struct MosquittoArc {
    inner: Arc<Mutex<Mosquitto>>
}

impl Clone for MosquittoArc {
    fn clone(&self) -> Self {
        MosquittoArc {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl MosquittoArc {
    pub fn new(host: impl Into<String>, port: u16, user: impl Into<String>, id: impl Into<String>) -> MosquittoArc {
        let (sender, receiver): (Sender<Mosquitto>, Receiver<Mosquitto>) = mpsc::channel();
        let host = host.into();
        let user = user.into();
        let id = id.into();
        let handle = Handle::current();
        thread::spawn(move || {
            let (mqtt, future) = Mosquitto::new(host, port, handle, user, id);
            sender.send(mqtt);
            block_on(future)
        });
        MosquittoArc {
            inner: Arc::new(Mutex::new(receiver.recv().unwrap())),
        }
    }

    delegate! {
        to self.inner.lock().unwrap() {
            pub fn subscribe(&mut self, topic: impl Into<String>, callback: impl Fn(&String, &String) + Send + 'static);
            pub fn unsubscribe(&mut self, topic: impl Into<String>);
            pub fn publish(&self, topic: impl Into<String>, payload: impl Into<String>);
            pub fn retain(&self, topic: impl Into<String>, payload: impl Into<String>);
            pub fn clear(&self, topic: impl Into<String>);
            pub fn stop(&mut self);
        }
    }

    pub async fn await_topic(
        &mut self, topic: impl Into<String>
    ) -> (String, String) {
        self.inner.lock().unwrap().await_topic(topic).await
    }

    pub async fn await_response(
        &mut self, topic_out: impl Into<String>, payload_out: impl Into<String>, topic_in: impl Into<String>
    ) -> (String, String) {
        self.inner.lock().unwrap().await_response(topic_out, payload_out, topic_in).await
    }
}

impl Mosquitto {
    pub fn new(
        host: impl Into<String>, port: u16, tokio_handle: Handle, user: impl Into<String>, id: impl Into<String>
    ) -> (Mosquitto, Join3<JoinHandle<()>, impl Future<Output=()>, JoinHandle<()>>) {
        let user = user.into();
        let mut mqttoptions = MqttOptions::new(id.into(), host.into(), port);
        mqttoptions.set_credentials(user, String::from(""));
        mqttoptions.set_keep_alive(5);

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        let callbacks = Arc::new(Mutex::new(HashMap::<String, (Regex, Vec<Box<dyn Fn(&String, &String) + Send + 'static>>)>::new()));
        let callbacks_for_thread = callbacks.clone();
        let callbacks_for_poller = callbacks.clone();
        let (event_sender, event_receiver): (Sender<(String, String)>, Receiver<(String, String)>) = mpsc::channel();
        let event_sender = event_sender.clone();

        let running = Arc::new(Mutex::from(true));
        let running_for_eventloop = Arc::clone(&running);
        let running_for_poller1 = Arc::clone(&running);
        let running_for_poller2 = Arc::clone(&running);

        let (subscription_sender, subscription_receiver): (Sender<Request>, Receiver<Request>) = mpsc::channel();

        (
            Mosquitto {
                running: running,
                sender: subscription_sender,
            },
            join3(
                tokio_handle.spawn(async move {
                    while *running_for_eventloop.lock().unwrap() {
                        let event = eventloop.poll().await;
                        match event {
                            Ok(event) => match event {
                                Event::Incoming(incoming) => match incoming {
                                    Packet::Publish(publish) => match String::from_utf8(publish.payload.to_vec()) {
                                        Ok(payload) => event_sender.send((publish.topic.clone(), payload.clone())).unwrap(),
                                        _ => {}
                                    }
                                    _ => {}
                                }
                                _ => {}
                            }
                            _ => {}
                        }
                    }
                }),
                async move {
                    while *running_for_poller1.lock().unwrap() {
                        match subscription_receiver.recv() {
                            Ok(request) => {
                                match request {
                                    Subscription(topic, callback) => {
                                        info!("Subscribing to topic: {}", &topic);
                                        let mut callbacks = callbacks_for_poller.lock().unwrap();
                                        match callbacks.get_mut(&topic) {
                                            Some((_, callbacks)) => callbacks.push(callback),
                                            None => {
                                                callbacks.insert(
                                                    topic.clone(),
                                                    (
                                                        Regex::new(topic.clone()
                                                            .replace("+", "[^/]+")
                                                            .replace("#", ".+").as_str())
                                                            .unwrap(),
                                                        vec![callback]
                                                    )
                                                );
                                            },
                                        }
                                        client.subscribe(&topic, QoS::ExactlyOnce).await.unwrap();
                                    }
                                    Publication(topic, payload, retain) => {
                                        info!("Publishing to topic: {}, the following: {}", topic, payload);
                                        client.publish(topic, QoS::ExactlyOnce, retain, payload).await.unwrap();
                                    }
                                    Unsubscription(topic) => {
                                        callbacks_for_poller.lock().unwrap().remove(&topic);
                                        client.unsubscribe(topic).await.unwrap();
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                },
                tokio_handle.spawn(async move {
                    while *running_for_poller2.lock().unwrap() {
                        match event_receiver.recv() {
                            Ok((topic, payload)) => {
                                info!("Got a event!");
                                for (key, (regex, callbacks)) in callbacks_for_thread.lock().unwrap().iter() {
                                    if key == &topic || regex.is_match(topic.as_str()) {
                                        for callback in callbacks {
                                            callback(&topic, &payload);
                                        }
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                })
            )
        )
    }

    pub fn subscribe(&mut self, topic: impl Into<String>, callback: impl Fn(&String, &String) + Send + 'static) {
        self.sender.send(Subscription(topic.into(), Box::new(callback))).unwrap();
    }

    pub fn unsubscribe(&mut self, topic: impl Into<String>) {
        self.sender.send(Unsubscription(topic.into())).unwrap();
    }

    pub fn publish(&self, topic: impl Into<String>, payload: impl Into<String>) {
        self.sender.send(Publication(topic.into(), payload.into(), false)).unwrap();
    }

    pub fn retain(&self, topic: impl Into<String>, payload: impl Into<String>) {
        self.sender.send(Publication(topic.into(), payload.into(), true)).unwrap();
    }

    pub fn clear(&self, topic: impl Into<String>) {
        self.sender.send(Publication(topic.into(), String::new(), true)).unwrap();
    }

    pub async fn await_topic(
        &mut self, topic: impl Into<String>
    ) -> (String, String) {
        let (sender, receiver): (Sender<(String, String)>, Receiver<(String, String)>) = mpsc::channel();

        let topic_in = topic.into();
        self.subscribe(&topic_in, move |topic, payload| {
            sender.send((topic.clone(), payload.clone()));
        });

        let result = receiver.recv().unwrap();
        self.unsubscribe(&topic_in);
        result
    }

    pub async fn await_response(
        &mut self, topic_out: impl Into<String>, payload_out: impl Into<String>, topic_in: impl Into<String>
    ) -> (String, String) {
        self.publish(topic_out, payload_out);
        self.await_topic(topic_in).await
    }

    pub fn stop(&mut self) {
        match self.running.lock() {
            Ok(mut running) => *running = false,
            Err(error) => error!("Error while stopping mqtt: {:?}", error)
        }
    }
}
