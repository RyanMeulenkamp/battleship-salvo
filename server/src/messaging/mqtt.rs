use crate::messaging::mqtt::Request::{Subscription, Publication};

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

pub enum Request {
    Subscription(String, Box<dyn FnMut(&String, &String) + Send + 'static>),
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
    pub fn new(
        id: impl Into<String>, host: impl Into<String>, port: u16, user: impl Into<String>
    ) -> (MosquittoArc, Join3<JoinHandle<()>, impl Future<Output=()>, JoinHandle<()>>) {
        let (inner, future) = Mosquitto::new(id, host, port, user);
        (
            MosquittoArc {
                inner: Arc::new(Mutex::new(inner)),
            },
            future
        )
    }

    delegate! {
        to self.inner.lock().unwrap() {
            pub fn subscribe(&mut self, topic: impl Into<String>, callback: impl FnMut(&String, &String) + Send + 'static);
            pub fn unsubscribe(&mut self, topic: impl Into<String>);
            pub fn publish(&self, topic: impl Into<String>, payload: impl Into<String>);
            pub fn retain(&self, topic: impl Into<String>, payload: impl Into<String>);
            pub fn clear(&self, topic: impl Into<String>);
            pub fn stop(&mut self);
        }
    }
}

impl Mosquitto {
    pub fn new(
        id: impl Into<String>, host: impl Into<String>, port: u16, user: impl Into<String>
    ) -> (Mosquitto, Join3<JoinHandle<()>, impl Future<Output=()>, JoinHandle<()>>) {
        let user = user.into();
        let mut mqttoptions = MqttOptions::new(id, host.into(), port);
        mqttoptions.set_credentials(user, String::from(""));
        mqttoptions.set_keep_alive(5);

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        let callbacks = Arc::new(Mutex::new(HashMap::<String, Vec<Box<dyn FnMut(&String, &String) + Send + 'static>>>::new()));
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
                task::spawn(async move {
                    while *running_for_eventloop.lock().unwrap() {
                        if let Ok(event) = eventloop.poll().await {
                            if let Event::Incoming(incoming) = event {
                                if let Packet::Publish(publish) = incoming {
                                    if let Ok(payload) = String::from_utf8(publish.payload.to_vec()) {
                                        event_sender.send((publish.topic.clone(), payload.clone())).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }),
                async move {
                    while *running_for_poller1.lock().unwrap() {
                        if let Ok(request) = subscription_receiver.recv() {
                            match request {
                                Subscription(topic, callback) => {
                                    info!("Subscribing to topic: {}", &topic);
                                    let mut callbacks = callbacks_for_poller.lock().unwrap();
                                    match callbacks.get_mut(&topic) {
                                        Some(callbacks) => callbacks.push(callback),
                                        None => {
                                            callbacks.insert(topic.clone(), vec![callback]);
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
                    }
                },
                task::spawn(async move {
                    while *running_for_poller2.lock().unwrap() {
                        if let Ok((topic, payload)) = event_receiver.recv() {
                            info!("Got a event!");
                            if let Some(callbacks) = callbacks_for_thread.lock().unwrap().get_mut(&topic) {
                                for callback in callbacks {
                                    callback(&topic, &payload);
                                }
                            }
                        }
                    }
                })
            )
        )
    }

    pub fn subscribe(&mut self, topic: impl Into<String>, callback: impl FnMut(&String, &String) + Send + 'static) {
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

    pub fn stop(&mut self) {
        match self.running.lock() {
            Ok(mut running) => *running = false,
            Err(error) => error!("Error while stopping mqtt: {:?}", error)
        }
    }
}
