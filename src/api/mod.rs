use serde::Serialize;
use crate::blockchain::Blockchain;
use crate::miner::Handle as MinerHandle;
use crate::network::server::Handle as NetworkServerHandle;
use crate::network::message::Message;
use crate::generator::generator::TransactionGenerator;
use crate::types::hash::{Hashable, H256};
use crate::types::state::State;
//use crate::blockchain::Blockchain;

use log::info;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::thread;
use tiny_http::Header;
use tiny_http::Response;
use tiny_http::Server as HTTPServer;
use url::Url;



pub struct Server {
    handle: HTTPServer,
    miner: MinerHandle,
    network: NetworkServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
    transaction_generator: TransactionGenerator, // Add transaction generator
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

macro_rules! respond_result {
    ( $req:expr, $success:expr, $message:expr ) => {{
        let content_type = "Content-Type: application/json".parse::<Header>().unwrap();
        let payload = ApiResponse {
            success: $success,
            message: $message.to_string(),
        };
        let resp = Response::from_string(serde_json::to_string_pretty(&payload).unwrap())
            .with_header(content_type);
        $req.respond(resp).unwrap();
    }};
}
macro_rules! respond_json {
    ( $req:expr, $message:expr ) => {{
        let content_type = "Content-Type: application/json".parse::<Header>().unwrap();
        let resp = Response::from_string(serde_json::to_string(&$message).unwrap())
            .with_header(content_type);
        $req.respond(resp).unwrap();
    }};
}

impl Server {
    pub fn start(
        addr: std::net::SocketAddr,
        miner: &MinerHandle,
        network: &NetworkServerHandle,
        blockchain: &Arc<Mutex<Blockchain>>,
        transaction_generator: &TransactionGenerator, // Pass transaction generator here 
    ) {
        let handle = HTTPServer::http(&addr).unwrap();
        let server = Self {
            handle,
            miner: miner.clone(),
            network: network.clone(),
            blockchain: Arc::clone(blockchain),
            transaction_generator: transaction_generator.clone(), // Clone transaction generator 
        };
        thread::spawn(move || {
            for req in server.handle.incoming_requests() {
                let miner = server.miner.clone();
                let network = server.network.clone();
                let blockchain = Arc::clone(&server.blockchain);
                let transaction_generator = server.transaction_generator.clone();
                thread::spawn(move || {
                    // a valid url requires a base
                    let base_url = Url::parse(&format!("http://{}/", &addr)).unwrap();
                    let url = match base_url.join(req.url()) {
                        Ok(u) => u,
                        Err(e) => {
                            respond_result!(req, false, format!("error parsing url: {}", e));
                            return;
                        }
                    };
                    match url.path() {
                        "/miner/start" => {
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let lambda = match params.get("lambda") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing lambda");
                                    return;
                                }
                            };
                            let lambda = match lambda.parse::<u64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("error parsing lambda: {}", e)
                                    );
                                    return;
                                }
                            };
                            miner.start(lambda);
                            respond_result!(req, true, "ok");
                        }
                        "/tx-generator/start" => {
                            // unimplemented!()
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let theta = match params.get("theta") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing theta");
                                    return;
                                }
                            };

                            let theta = match theta.parse::<u64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("error parsing theta: {}", e)
                                    );
                                    return;
                                }
                            };

                            transaction_generator.start(theta);
                            //respond_result!(req, false, "unimplemented!");
                            respond_result!(req, true, "Transaction generator started");
                        }
                        "/network/ping" => {
                            network.broadcast(Message::Ping(String::from("Test ping")));
                            respond_result!(req, true, "ok");
                        }
                        "/blockchain/longest-chain" => {
                            let blockchain = blockchain.lock().unwrap();
                            let v = blockchain.all_blocks_in_longest_chain();
                            let v_string: Vec<String> = v.into_iter().map(|h|h.to_string()).collect();
                            respond_json!(req, v_string);
                            drop(blockchain);
                        }
                        "/blockchain/longest-chain-tx" => {
                            // unimplemented!()
                            let blockchain = blockchain.lock().unwrap();
                            let longest_chain = blockchain.all_blocks_in_longest_chain();
                            let mut tx_chain: Vec<Vec<String>> = Vec::new();

                            for block_hash in longest_chain {
                                if let Some(block) = blockchain.blocks.get(&block_hash) {
                                    let tx_hashes: Vec<String> = block
                                        .content
                                        .transactions
                                        .iter()
                                        .map(|tx| tx.hash().to_string())
                                        .collect();
                                    tx_chain.push(tx_hashes);
                                } else {
                                    tx_chain.push(vec![]);
                                }
                            }
                            respond_json!(req, tx_chain);
                            drop(blockchain);
                            //respond_result!(req, false, "unimplemented!");
                        }
                        "/blockchain/longest-chain-tx-count" => {
                            // unimplemented!()
                            respond_result!(req, false, "unimplemented!");
                        }
                        "/blockchain/state" => {
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let block_param = match params.get("block") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing block parameter");
                                    return;
                                }
                            };
                            
                            let block_index = match block_param.parse::<usize>() {
                                Ok(index) => index,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("Invalid block index: {}", e)
                                    );
                                    return;
                                }

                            };


                            let blockchain = blockchain.lock().unwrap();
                            let longest_chain = blockchain.all_blocks_in_longest_chain();

                            if block_index >= longest_chain.len() {
                                respond_result!(
                                    req,
                                    false,
                                    format!("block index {} exceeds the longest chain length", block_index)
                                );
                                return;
                            }

                            let block_hash = longest_chain[block_index];

                            let state_map = blockchain.states.clone();

                            if let Some(state) = state_map.get(&block_hash) {
                                let state = state.lock().unwrap();
                                let state_representation: Vec<String> = state
                                    .get_state_snapshot()
                                    .into_iter()
                                    .map(|(address, (nonce, balance))| format!("({}, {}, {})", address, nonce, balance))
                                    .collect();
                                respond_json!(req, state_representation);
                                drop(state);
                            } else {
                                respond_result!(
                                    req,
                                    false,
                                    format!("State not found for block: {}", block_hash)
                                );
                            }
                            drop(blockchain);
                        }
                        _ => {
                            let content_type =
                                "Content-Type: application/json".parse::<Header>().unwrap();
                            let payload = ApiResponse {
                                success: false,
                                message: "endpoint not found".to_string(),
                            };
                            let resp = Response::from_string(
                                serde_json::to_string_pretty(&payload).unwrap(),
                            )
                            .with_header(content_type)
                            .with_status_code(404);
                            req.respond(resp).unwrap();
                        }
                    }
                });
            }
        });
        info!("API server listening at {}", &addr);
    }
}
