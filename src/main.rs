#[macro_use]
extern crate log;

use bytes::Bytes;
use ckb_vm::{
    machine::asm::{AsmCoreMachine, AsmMachine},
    DefaultMachineBuilder, SupportMachine,
};
use ckb_vm_debug_utils::{GdbHandler, Stdio};
use gdb_remote_protocol::process_packets_from;
use std::env;
use std::fs::File;
use std::io::Read;
use std::net::TcpListener;

fn main() {
    drop(env_logger::init());
    let args: Vec<String> = env::args().skip(1).collect();

    let listener = TcpListener::bind(&args[0]).expect("listen");
    debug!("Listening on {}", args[0]);

    let mut file = File::open(&args[1]).expect("open program");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let program: Bytes = buffer.into();
    let program_args: Vec<Bytes> = args.into_iter().skip(1).map(|a| a.into()).collect();

    for res in listener.incoming() {
        debug!("Got connection");
        if let Ok(stream) = res {
            let core = DefaultMachineBuilder::<Box<AsmCoreMachine>>::default()
                .syscall(Box::new(Stdio::new(true)))
                .build();
            let mut machine = AsmMachine::new(core, None);
            machine
                .load_program(&program, &program_args)
                .expect("load program");
            machine.machine.set_running(true);
            let h = GdbHandler::new(machine);
            process_packets_from(stream.try_clone().unwrap(), stream, h);
        }
        debug!("Connection closed");
    }
}
