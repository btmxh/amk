use std::{
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread::{self, JoinHandle},
};

use super::{
    loops::{GameLoop, GameLoopContainer, GameLoopKind},
    msg::{FromRunnerMsg, ToRunnerMsg},
};

pub(crate) struct Runner {
    thread_handle: JoinHandle<anyhow::Result<()>>,
    sender: Sender<ToRunnerMsg>,
    receiver: Receiver<FromRunnerMsg>,
}

impl Runner {
    pub fn new() -> Self {
        let (f_sender, f_receiver) = mpsc::channel::<FromRunnerMsg>();
        let (t_sender, t_receiver) = mpsc::channel::<ToRunnerMsg>();
        Self {
            thread_handle: thread::spawn(move || {
                let sender = f_sender;
                let receiver = t_receiver;
                let mut container = GameLoopContainer::new();
                loop {
                    if let Some(msg) = Self::receive_msg(&receiver, container.empty()) {
                        match msg {
                            ToRunnerMsg::Stop => break,
                            ToRunnerMsg::RequestLoop(kind) => sender
                                .send(FromRunnerMsg::SendLoop(container.get(kind).unwrap()))
                                .unwrap(),
                            ToRunnerMsg::SendLoop(kind, gl, relative_frequency) => {
                                container.insert(kind, gl, relative_frequency)
                            }
                            ToRunnerMsg::SetRelativeFrequency(kind, relative_frequency) => {
                                container.set_relative_frequency(kind, relative_frequency)
                            }
                        }
                    }
                }
                anyhow::Ok(())
            }),
            sender: t_sender,
            receiver: f_receiver,
        }
    }

    fn receive_msg(recv: &Receiver<ToRunnerMsg>, block: bool) -> Option<ToRunnerMsg> {
        if block {
            Some(recv.recv().unwrap())
        } else {
            match recv.try_recv() {
                Err(TryRecvError::Empty) => None,
                e => Some(e.unwrap()),
            }
        }
    }

    pub(crate) fn request_loop(&self, kind: GameLoopKind) -> Box<dyn GameLoop> {
        self.sender.send(ToRunnerMsg::RequestLoop(kind)).unwrap();
        let msg = self.receiver.recv().unwrap();
        if let FromRunnerMsg::SendLoop(gl) = msg {
            gl
        } else {
            panic!("invalid response")
        }
    }

    pub(crate) fn send_loop(
        &self,
        kind: GameLoopKind,
        gl: Box<dyn GameLoop>,
        relative_frequency: f64,
    ) {
        self.sender
            .send(ToRunnerMsg::SendLoop(kind, gl, relative_frequency))
            .unwrap();
    }

    pub(crate) fn set_relative_frequency(&self, kind: GameLoopKind, relative_frequency: f64) {
        self.sender
            .send(ToRunnerMsg::SetRelativeFrequency(kind, relative_frequency))
            .unwrap();
    }
}
