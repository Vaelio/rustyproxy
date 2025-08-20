use rusqlite::Connection;
use service::{HistoryEntry, InspectorEntry, ApiService, OrderByEnum};
use tarpc::{
    context,
    server::{self, incoming::Incoming, Channel},
    tokio_serde::formats::Bincode,
};
use std::{
    net::{IpAddr, Ipv4Addr},
    sync::{Arc, Mutex},
};
use futures::{future, prelude::*};

#[derive(Clone)]
struct RpcServer {
    conn: Arc<Mutex<Connection>>,
}

impl ApiService for RpcServer {
    async fn get_history_entry(self, _: context::Context, id: usize) -> Option<HistoryEntry> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM history WHERE id == ?").unwrap();
        let row = stmt.query_row([id], |row| {
            let raw: String = row.get(8).unwrap();

            Ok(HistoryEntry{
                id: row.get(0).unwrap(),
                remote_addr: row.get(1).unwrap(),
                uri: row.get(2).unwrap(),
                method: row.get(3).unwrap(),
                params: matches!(row.get(4).unwrap(), 1),
                status: row.get(5).unwrap(),
                size: row.get(6).unwrap(),
                timestamp: row.get(7).unwrap(),
                raw: raw.into_bytes(),
                ssl: row.get(9).unwrap(),
                response: row.get(10).unwrap(),
                response_time: row.get(11).unwrap(),
                content_length: row.get(12).unwrap(),
            })
        });

        row.ok()
    }

    async fn list_history_entries(self, _: context::Context, page: usize, page_size: usize, order: OrderByEnum) -> Vec<HistoryEntry> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare(&format!("SELECT * FROM history ORDER BY ID {} LIMIT ? OFFSET ?", order.to_string())) {
            Ok(v) => v,
            Err(e) => panic!("ERROR: {:#?}", e),
        };
        let rows = stmt.query_map([page_size, page], |row| {
            let raw: String = row.get(8).unwrap();

            Ok(HistoryEntry{
                id: row.get(0).unwrap(),
                remote_addr: row.get(1).unwrap(),
                uri: row.get(2).unwrap(),
                method: row.get(3).unwrap(),
                params: matches!(row.get(4).unwrap(), 1),
                status: row.get(5).unwrap(),
                size: row.get(6).unwrap(),
                timestamp: row.get(7).unwrap(),
                raw: raw.into_bytes(),
                ssl: row.get(9).unwrap(),
                response: row.get(10).unwrap(),
                response_time: row.get(11).unwrap(),
                content_length: row.get(12).unwrap(),
            })
        }).unwrap();

        rows.filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect::<Vec<HistoryEntry>>()
    }

    async fn count_history_entries(self, _: context::Context) -> usize {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare("SELECT COUNT(*) FROM history") {
            Ok(v) => v,
            Err(e) => panic!("ERROR: {:#?}", e),
        };
        stmt.query_one([], |row|{
            let ret: usize = row.get(0).unwrap();
            Ok(ret)
        }).unwrap()
    }

    async fn get_inspector_entry(self, _: context::Context, id: usize) -> Option<InspectorEntry> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM inspectors WHERE id == ?").unwrap();
        let row = stmt.query_row([id], |row| {
            Ok(InspectorEntry{
                id: row.get(0).unwrap(),
                request: row.get(1).unwrap(),
                response: row.get(2).unwrap(),
                modified_request: row.get(3).unwrap(),
                new_response: row.get(4).unwrap(),
                ssl: matches!(row.get(5).unwrap(), 1),
                target: row.get(6).unwrap()
            })
        });

        row.ok()
    }

    async fn list_inspector_entries(self, _: context::Context, page: usize, page_size: usize, order: OrderByEnum) -> Vec<InspectorEntry> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare(&format!("SELECT * FROM inspectors ORDER BY ID {} LIMIT ? OFFSET ?", order.to_string())) {
            Ok(v) => v,
            Err(e) => panic!("ERROR: {:#?}", e),
        };

        let rows = stmt.query_map([page_size, page], |row| {

            Ok(InspectorEntry{
                id: row.get(0).unwrap(),
                request: row.get(1).unwrap(),
                response: row.get(2).unwrap(),
                modified_request: row.get(3).unwrap(),
                new_response: row.get(4).unwrap(),
                ssl: matches!(row.get(5).unwrap(), 1),
                target: row.get(6).unwrap()
            })
        }).unwrap();

        rows.filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect::<Vec<InspectorEntry>>()
    }
    
    async fn count_inspector_entries(self, _: context::Context) -> usize {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare("SELECT COUNT(*) FROM inspectors") {
            Ok(v) => v,
            Err(e) => panic!("ERROR: {:#?}", e),
        };
        stmt.query_one([], |row|{
            let ret: usize = row.get(0).unwrap();
            Ok(ret)
        }).unwrap()
    }
}

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

pub async fn handle_rpc() -> anyhow::Result<()> {
    let server_addr = (IpAddr::V4(Ipv4Addr::new(0,0,0,0)), 1337);

    // JSON transport is provided by the json_transport tarpc module. It makes it easy
    // to start up a serde-powered json serialization strategy over TCP.
    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Bincode::default).await?;
    tracing::info!("Listening on port {}", listener.local_addr().port());
    listener.config_mut().max_frame_length(usize::MAX);
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        // Limit channels to 1 per IP.
        .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
        // serve is generated by the service attribute. It takes as input any type implementing
        // the generated World trait.
        .map(|channel| {
            let conn = Connection::open("hist.db").unwrap();
            conn.execute_batch("PRAGMA journal_mode=WAL;").unwrap();
            let conn = Arc::new(Mutex::new(conn));
            let server = RpcServer{ conn };
            channel.execute(server.serve()).for_each(spawn)
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}