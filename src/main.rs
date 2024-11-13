#[cfg(test)]
#[macro_use]
extern crate hex_literal;

pub mod api;
pub mod blockchain;
pub mod types;
pub mod miner;
pub mod network;
pub mod generator;

use blockchain::Blockchain;
use clap::clap_app;
use smol::channel;
use log::{error, info};
use api::Server as ApiServer;
use std::net;
use std::process;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use types::transaction::Mempool;

fn main() {
    // parse command line arguments
    let matches = clap_app!(Bitcoin =>
     (version: "0.1")
     (about: "Bitcoin client")
     (@arg verbose: -v ... "Increases the verbosity of logging")
     (@arg peer_addr: --p2p [ADDR] default_value("127.0.0.1:6000") "Sets the IP address and the port of the P2P server")
     (@arg api_addr: --api [ADDR] default_value("127.0.0.1:7000") "Sets the IP address and the port of the API server")
     (@arg known_peer: -c --connect ... [PEER] "Sets the peers to connect to at start")
     (@arg p2p_workers: --("p2p-workers") [INT] default_value("4") "Sets the number of worker threads for P2P server")
    )
    .get_matches();

    // init logger
    let verbosity = matches.occurrences_of("verbose") as usize;
    stderrlog::new().verbosity(verbosity).init().unwrap();
    let blockchain = Blockchain::new();
    let blockchain = Arc::new(Mutex::new(blockchain));

    //Initialize the Mempool
    let mempool = Mempool::new(1000); // Set max transactions in mempool as needed
    let mempool = Arc::new(Mutex::new(mempool));

    // parse p2p server address
    let p2p_addr = matches
        .value_of("peer_addr")
        .unwrap()
        .parse::<net::SocketAddr>()
        .unwrap_or_else(|e| {
            error!("Error parsing P2P server address: {}", e);
            process::exit(1);
        });

    // parse api server address
    let api_addr = matches
        .value_of("api_addr")
        .unwrap()
        .parse::<net::SocketAddr>()
        .unwrap_or_else(|e| {
            error!("Error parsing API server address: {}", e);
            process::exit(1);
        });

    // create channels between server and worker
    let (msg_tx, msg_rx) = channel::bounded(10000);

    // start the p2p server
    let (server_ctx, server) = network::server::new(p2p_addr, msg_tx).unwrap();
    server_ctx.start().unwrap();

    // start the worker
    let p2p_workers = matches
        .value_of("p2p_workers")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing P2P workers: {}", e);
            process::exit(1);
        });
    let worker_ctx = network::worker::Worker::new(
        p2p_workers,
        msg_rx,
        &server,
        &blockchain, // Pass the shared blockchain to the network worker
        &mempool, // Pass the shared mempool to the network server
    );
    worker_ctx.start();

    // start the miner
    let (miner_ctx, miner, finished_block_chan) = miner::new(&blockchain, &mempool);
    let miner_worker_ctx = miner::worker::Worker::new(&server, finished_block_chan, &blockchain, &mempool, 10); // Assuming 10 as max transactions per block
    miner_ctx.start();
    miner_worker_ctx.start();

    // Initialize the transaction generator with mempool and start it
    let transaction_generator = generator::generator::TransactionGenerator::new(mempool.clone(), server.clone());
    //transaction_generator.clone().start(100); // Example 'theta' value; adjust as needed 

    // connect to known peers
    if let Some(known_peers) = matches.values_of("known_peer") {
        let known_peers: Vec<String> = known_peers.map(|x| x.to_owned()).collect();
        let server = server.clone();
        thread::spawn(move || {
            for peer in known_peers {
                loop {
                    let addr = match peer.parse::<net::SocketAddr>() {
                        Ok(x) => x,
                        Err(e) => {
                            error!("Error parsing peer address {}: {}", &peer, e);
                            break;
                        }
                    };
                    match server.connect(addr) {
                        Ok(_) => {
                            info!("Connected to outgoing peer {}", &addr);
                            break;
                        }
                        Err(e) => {
                            error!(
                                "Error connecting to peer {}, retrying in one second: {}",
                                addr, e
                            );
                            thread::sleep(time::Duration::from_millis(1000));
                            continue;
                        }
                    }
                }
            }
        });
    }


    // start the transaction generator
    //let transaction_generator = generator::generator::TransactionGenerator::new(Arc::clone(&mempool), server.clone());

    // start the API server
    ApiServer::start(
        api_addr,
        &miner,
        &server,
        &blockchain,
        &transaction_generator, // Pass the transaction generator
    );

    loop {
        std::thread::park();
    }
}



