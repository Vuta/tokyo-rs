use crate::actors::ClientWsActor;
use crate::game::Game;
use crate::models::messages::PlayerGameCommand;
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use common::models::*;
use futures::sync::oneshot;
use spin_sleep::LoopHelper;
use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub struct GameActor {
    connections: HashSet<Addr<ClientWsActor>>,
    cancel_chan: Option<oneshot::Sender<()>>,
    msg_tx: Sender<GameCommand>,
    msg_rx: Option<Receiver<GameCommand>>,
}

impl GameActor {
    pub fn new() -> GameActor {
        let (msg_tx, msg_rx) = channel();

        GameActor {
            connections: HashSet::new(),
            cancel_chan: None,
            msg_tx,
            msg_rx: Some(msg_rx),
        }
    }
}

fn game_loop(
    game_actor: Addr<GameActor>,
    msg_chan: Receiver<GameCommand>,
    mut cancel_chan: oneshot::Receiver<()>,
) {
    let target_update_per_second = 30;

    let mut loop_helper = LoopHelper::builder().build_with_target_rate(target_update_per_second);

    let game = Game::new();

    game.init();

    loop {
        loop_helper.loop_start();

        match cancel_chan.try_recv() {
            Ok(Some(_)) | Err(_) => {
                break;
            }
            _ => {}
        }

        for cmd in msg_chan.try_iter() {
            println!("Got a message! - {:?}", cmd);
        }

        let dt = 1.0 / target_update_per_second as f32;
        game.tick(dt);

        // Send out update packets

        // TODO(bschwind) - maybe put the game state behind an Arc
        //                  instead of cloning it
        game_actor.do_send(game.state.clone());
        loop_helper.loop_sleep();
    }

    println!("game over!");
}

impl Actor for GameActor {
    type Context = Context<GameActor>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Game Actor started!");
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let addr = ctx.address();

        // "Take" the receiving end of the channel and give it
        // to the game loop thread
        let msg_rx = self.msg_rx.take().unwrap();

        std::thread::spawn(move || {
            game_loop(addr, msg_rx, cancel_rx);
        });

        self.cancel_chan = Some(cancel_tx);
    }
}

#[derive(Message)]
pub enum SocketEvent {
    Join(Addr<ClientWsActor>),
    Leave(Addr<ClientWsActor>),
}

impl Handler<SocketEvent> for GameActor {
    type Result = ();

    fn handle(&mut self, msg: SocketEvent, _ctx: &mut Self::Context) {
        match msg {
            SocketEvent::Join(addr) => {
                println!("person joined - {:?}", addr);
                self.connections.insert(addr);
            }
            SocketEvent::Leave(addr) => {
                println!("person left - {:?}", addr);
                self.connections.remove(&addr);
            }
        }
    }
}

impl Handler<PlayerGameCommand> for GameActor {
    type Result = ();

    fn handle(&mut self, msg: PlayerGameCommand, _ctx: &mut Self::Context) {
        self.msg_tx
            .send(msg.cmd)
            .expect("The game loop should always be receiving commands");
    }
}

impl Handler<GameState> for GameActor {
    type Result = ();

    fn handle(&mut self, msg: GameState, _ctx: &mut Self::Context) {
        for addr in self.connections.iter() {
            addr.do_send(msg.clone());
        }
    }
}
