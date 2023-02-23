//! start the bootstrapping system using [`start_bootstrap_server`]
//! Once your node will be ready, you may want other to bootstrap from you.
//!
//! # Listener
//!
//! Runs in the server-dedication tokio async runtime
//! Accepts bootstrap connections in an async-loop
//! Upon connection, pushes the accepted connection onto a channel for the worker loop to consume
//!
//! # Updater
//!
//! Runs in the same runtime as the listener.
//! Shares an Arc<RwLock>> guarded list of white and blacklists with the main worker.
//! Periodically does a read-only check to see if list needs updating.
//! Creates an updated list then swaps it out with write-locked list
//! Assuming no errors in code, this is the only write occurance, and is only a pointer-swap
//! under the hood, making write contention virtually non-existant.
//!
//! # Worker loop
//!
//! 1. Checks if the stopper has been invoked.
//! 2. Checks if the client is permited under the white/black list rules
//! 3. Checks if there are not too many active sessions already
//! 4. Checks if the client has attempted too recently
//! 5. All checks have passed: spawn a thread on which to run the bootstrap session
//!    This thread creates a new tokio runtime, and runs it with `block_on`
mod white_black_list;
use white_black_list::*;

use crossbeam::channel::{Receiver, Select, SendError, Sender};
use humantime::format_duration;
use massa_async_pool::AsyncMessageId;
use massa_consensus_exports::{bootstrapable_graph::BootstrapableGraph, ConsensusController};
use massa_final_state::{FinalState, FinalStateError};
use massa_logging::massa_trace;
use massa_models::{
    block_id::BlockId, prehash::PreHashSet, slot::Slot, streaming_step::StreamingStep,
    version::Version,
};
use massa_network_exports::NetworkCommandSender;
use massa_signature::KeyPair;
use massa_time::MassaTime;
use parking_lot::RwLock;
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
use tokio::runtime::Runtime;
use tracing::{debug, info, warn};

use crate::{
    error::BootstrapError,
    messages::{BootstrapClientMessage, BootstrapServerMessage},
    server_binder::BootstrapServerBinder,
    types::{Duplex, Listener},
    BootstrapConfig, Establisher,
};

/// Abstraction layer over data produced by the listener, and transported
/// over to the worker via a channel
type BsConn = (Duplex, SocketAddr);
/// handle on the bootstrap server
pub struct BootstrapManager {
    update_handle: tokio::task::JoinHandle<Result<(), BootstrapError>>,
    listen_handle: tokio::task::JoinHandle<Result<Result<(), BsConn>, Box<BootstrapError>>>,
    main_handle: std::thread::JoinHandle<Result<(), Box<BootstrapError>>>,
    stopper_tx: crossbeam::channel::Sender<()>,
}

impl BootstrapManager {
    /// stop the bootstrap server
    pub fn stop(self) -> Result<(), Box<BootstrapError>> {
        massa_trace!("bootstrap.lib.stop", {});
        if self.stopper_tx.send(()).is_err() {
            warn!("bootstrap server already dropped");
        }
        self.listen_handle.abort();
        self.update_handle.abort();
        dbg!(self.main_handle.join().unwrap())
    }
}

/// See module level documentation for details
pub async fn start_bootstrap_server(
    consensus_controller: Box<dyn ConsensusController>,
    network_command_sender: NetworkCommandSender,
    final_state: Arc<RwLock<FinalState>>,
    config: BootstrapConfig,
    mut establisher: Establisher,
    keypair: KeyPair,
    version: Version,
) -> Result<Option<BootstrapManager>, Box<BootstrapError>> {
    massa_trace!("bootstrap.lib.start_bootstrap_server", {});
    let Some(listen_addr) = config.listen_addr else {
        return Ok(None);
    };

    // TODO(low prio): See if a zero capacity channel model can work
    let (stopper_tx, stopper_rx) = crossbeam::channel::bounded::<()>(1);

    let outer_server_runtime = Runtime::new().expect("Failed to create a bootstrap runtime");
    let listener = establisher
        .get_listener(listen_addr)
        .await
        .map_err(BootstrapError::IoError)?;

    let Ok(max_bootstraps) = config.max_simultaneous_bootstraps.try_into() else {
        return Err(BootstrapError::GeneralError("Fail to convert u32 to usize".to_string()).into());
    };
    // This is the primary interface between the async-listener, and the (soon to be) sync worker
    let (listener_tx, listener_rx) = crossbeam::channel::bounded::<BsConn>(max_bootstraps);

    let white_black_list = SharedWhiteBlackList::new(
        config.bootstrap_whitelist_path.clone(),
        config.bootstrap_blacklist_path.clone(),
    )?;

    let update_handle = outer_server_runtime
        .handle()
        .clone()
        .spawn(BootstrapServer::run_updater(
            white_black_list.clone(),
            config.cache_duration.into(),
        ));
    let listen_handle = outer_server_runtime
        .handle()
        .clone()
        .spawn(BootstrapServer::run_listener(listener, listener_tx));

    let main_handle = std::thread::spawn(move || {
        BootstrapServer {
            consensus_controller,
            network_command_sender,
            final_state,
            listener_rx,
            stopper_rx,
            white_black_list,
            keypair,
            version,
            ip_hist_map: HashMap::with_capacity(config.ip_list_max_size),
            bootstrap_config: config,
            outer_server_runtime,
        }
        .run_loop(max_bootstraps)
    });
    // Give the runtime to the bootstrap manager, otherwise it will be dropped, forcibly aborting the spawned tasks.
    // TODO: make the tasks sync, so the runtime is redundant
    Ok(Some(BootstrapManager {
        update_handle,
        listen_handle,
        main_handle,
        // Send on this channel to trigger the tokio::select! loop to break
        stopper_tx,
    }))
}

struct BootstrapServer<'a> {
    consensus_controller: Box<dyn ConsensusController>,
    network_command_sender: NetworkCommandSender,
    final_state: Arc<RwLock<FinalState>>,
    listener_rx: Receiver<BsConn>,
    stopper_rx: Receiver<()>,
    white_black_list: SharedWhiteBlackList<'a>,
    keypair: KeyPair,
    bootstrap_config: BootstrapConfig,
    version: Version,
    ip_hist_map: HashMap<IpAddr, Instant>,
    outer_server_runtime: Runtime,
}

impl BootstrapServer<'_> {
    async fn run_updater(
        mut list: SharedWhiteBlackList<'_>,
        interval: Duration,
    ) -> Result<(), BootstrapError> {
        let mut interval = tokio::time::interval(interval);
        loop {
            interval.tick().await;
            // TODO: loop interval here is tick + update time. Implement a state-based,
            // rather than time-based, trigger (such as delta-count);
            list.update()?;
        }
    }

    /// Listens on a channel for incoming connections, and loads them onto a crossbeam channel
    /// for the main-loop to process.
    ///
    /// Ok(Ok(())) listener closed without issue
    /// Ok(Err((dplx, address))) listener accepted a connection then tried sending on a disconnected channel
    /// Err(..) Error accepting a connection
    async fn run_listener(
        mut listener: Listener,
        listener_tx: Sender<BsConn>,
    ) -> Result<Result<(), BsConn>, Box<BootstrapError>> {
        loop {
            let msg = listener.accept().await.map_err(BootstrapError::IoError)?;
            match listener_tx.send(msg) {
                Ok(_) => continue,
                Err(SendError((dplx, remote_addr))) => {
                    warn!(
                        "listener channel disconnected after accepting connection from {}",
                        remote_addr
                    );
                    return Ok(Err((dplx, remote_addr)));
                }
            };
        }
    }

    fn run_loop(&mut self, max_bootstraps: usize) -> Result<(), Box<BootstrapError>> {
        // Use the strong-count of this variable to track the session count
        let bootstrap_sessions_counter: Arc<()> = Arc::new(());
        let per_ip_min_interval = self.bootstrap_config.per_ip_min_interval.to_duration();
        let mut selector = Select::new();
        selector.recv(&self.stopper_rx);
        selector.recv(&self.listener_rx);
        // TODO: Work out how to integration-test this
        loop {
            // block until we have a connection to work with, or break out of main-loop
            let Some((dplx, remote_addr)) = self.receive_connection(&mut selector).map_err(BootstrapError::GeneralError)? else {break};
            // claim a slot in the max_bootstrap_sessions
            let server = BootstrapServerBinder::new(
                dplx,
                self.keypair.clone(),
                (&self.bootstrap_config).into(),
            );

            // check whether incoming peer IP is allowed.
            // TODO: confirm error handled according to the previous `is_ip_allowed` fn
            if let Err(error_msg) = self.white_black_list.is_ip_allowed(&remote_addr) {
                server.close_and_send_error(
                    self.outer_server_runtime.handle().clone(),
                    error_msg.to_string(),
                    remote_addr,
                    move || {},
                );
                // not exactly sure what to do here
                // return Err(std::io::Error::new(
                //     std::io::ErrorKind::PermissionDenied,
                //     error_msg,
                // ));
                continue;
            };

            // TODO: find a better way to track count
            // the `- 1` is to account for the top-level Arc that is created at the top
            // of this method. subsequent counts correspond to each `clone` that is passed
            // into a thread
            // TODO: If we don't find a way to handle the counting automagically, make
            //       a dedicated wrapper-type with doc-comments, manual drop impl that
            //       integrates logging, etc...
            if Arc::strong_count(&bootstrap_sessions_counter) - 1 < max_bootstraps {
                massa_trace!("bootstrap.lib.run.select.accept", {
                    "remote_addr": remote_addr
                });
                let now = Instant::now();

                // clear IP history if necessary
                if self.ip_hist_map.len() > self.bootstrap_config.ip_list_max_size {
                    self.ip_hist_map
                        .retain(|_k, v| now.duration_since(*v) <= per_ip_min_interval);
                    if self.ip_hist_map.len() > self.bootstrap_config.ip_list_max_size {
                        // too many IPs are spamming us: clear cache
                        warn!("high bootstrap load: at least {} different IPs attempted bootstrap in the last {}", self.ip_hist_map.len(),format_duration(self.bootstrap_config.per_ip_min_interval.to_duration()).to_string());
                        self.ip_hist_map.clear();
                    }
                }

                // check IP's bootstrap attempt history
                if let Err(msg) = BootstrapServer::greedy_client_check(
                    &mut self.ip_hist_map,
                    remote_addr,
                    now,
                    per_ip_min_interval,
                ) {
                    let msg = format!(
                        "Your last bootstrap on this server was {} ago and you have to wait {} before retrying.",
                        format_duration(msg),
                        format_duration(per_ip_min_interval.saturating_sub(msg))
                    );
                    let tracer = move || {
                        massa_trace!("bootstrap.lib.run.select.accept.refuse_limit", {
                            "remote_addr": remote_addr
                        })
                    };
                    server.close_and_send_error(
                        self.outer_server_runtime.handle().clone(),
                        msg,
                        remote_addr,
                        tracer,
                    );
                    continue;
                };

                // load cache if absent
                // if bootstrap_data.is_none() {
                //     massa_trace!("bootstrap.lib.run.select.accept.cache_load.start", {});

                //     // Note that all requests are done simultaneously except for the consensus graph that is done after the others.
                //     // This is done to ensure that the execution bootstrap state is older than the consensus state.
                //     // If the consensus state snapshot is older than the execution state snapshot,
                //     //   the execution final ledger will be in the future after bootstrap, which causes an inconsistency.
                //     bootstrap_data = Some((data_graph, data_peers, self.final_state.clone()));
                //     cache_timer.set(sleep(cache_timeout));
                // }
                massa_trace!("bootstrap.lib.run.select.accept.cache_available", {});

                // launch bootstrap
                let version = self.version;
                let data_execution = self.final_state.clone();
                let consensus_command_sender = self.consensus_controller.clone();
                let network_command_sender = self.network_command_sender.clone();
                let config = self.bootstrap_config.clone();

                let bootstrap_count_token = bootstrap_sessions_counter.clone();
                let _ = thread::Builder::new()
                    .name(format!("bootstrap thread, peer: {}", remote_addr))
                    .spawn(move || {
                        run_bootstrap_session(
                            server,
                            bootstrap_count_token,
                            config,
                            remote_addr,
                            data_execution,
                            version,
                            consensus_command_sender,
                            network_command_sender,
                        )
                    });

                massa_trace!("bootstrap.session.started", {
                    "active_count": Arc::strong_count(&bootstrap_sessions_counter) - 1
                });
            } else {
                server.close_and_send_error(
                    self.outer_server_runtime.handle().clone(),
                    "Bootstrap failed because the bootstrap server currently has no slots available.".to_string(),
                    remote_addr,
                    move || debug!("did not bootstrap {}: no available slots", remote_addr)
                );
            }
        }
        // TODO: when they are synchrenous, consider if we want to signal updater and listener
        //       here, or in the stop method.
        // TODO: do we drop(self.runtime) here?
        Ok(())
    }

    /// this is a bit complicated, but the simple is bad:
    /// ```rust-ignore
    /// if self.stopper_rx.try_recv().is_ok() {
    ///     break;
    /// }
    /// // if the server is stopped while blocked here, this risks never unblocking
    /// // OR: would serve as an OBO error; Server doesn't actually stop working until after
    /// // it's done processing one more connection _after_ it's been stopped.
    /// // Any bug fix at that point would be a downstream hack to an upstream
    /// // bug.
    /// let Ok(msg) = self.listener_rx.recv() else {continue};
    /// ```
    ///
    /// The solution here is to:
    /// - 1. Block until _something_ is ready
    /// - 2. If that something is a stop-signal, stop
    /// - 3. If that something is anything else:
    /// - 3.a. double check the stop-signal is absent
    /// - 3.b. If present, fall-back to the stop behaviour
    /// - 3.c. If absent, all's clear to rock-n-roll.
    fn receive_connection(&self, selector: &mut Select) -> Result<Option<BsConn>, String> {
        // 1. Block until _something_ is ready
        let rdy = selector.ready();

        // 2. If that something is a stop-signal, stop
        // from `crossbeam::Select::read()` documentation:
        // "Note that this method might return with success spuriously, so it’s a good idea
        // to always double check if the operation is really ready."
        if rdy == 0 && self.stopper_rx.try_recv().is_ok() {
            return Ok(None);
            // - 3. If that something is anything else:
        } else {
            massa_trace!("bootstrap.lib.run.select", {});

            // - 3.a. double check the stop-signal is absent
            let stop = self.stopper_rx.try_recv();
            if unlikely(stop.is_ok()) {
                massa_trace!("bootstrap.lib.run.select.manager", {});
                return Ok(None);
            } else if unlikely(stop == Err(crossbeam::channel::TryRecvError::Disconnected)) {
                return Err("Unexpected stop-channel disconnection".to_string());
            };
        }
        // - 3.c. If absent, all's clear to rock-n-roll.
        let msg = match self.listener_rx.try_recv() {
            Ok(msg) => msg,
            Err(try_rcv_err) => match try_rcv_err {
                crossbeam::channel::TryRecvError::Empty => return Ok(None),
                crossbeam::channel::TryRecvError::Disconnected => {
                    return Err("listener recv channel disconnected unexpectedly".to_string())
                }
            },
        };
        Ok(Some(msg))
    }
    /// Helper method to check if this IP is being greedy, i.e. not enough elapsed time since last attempt
    ///
    /// # Error
    /// The elapsed time which is insufficient
    fn greedy_client_check(
        ip_hist_map: &mut HashMap<IpAddr, Instant>,
        remote_addr: SocketAddr,
        now: Instant,
        per_ip_min_interval: Duration,
    ) -> Result<(), Duration> {
        let mut res = Ok(());
        ip_hist_map
            .entry(remote_addr.ip())
            .and_modify(|occ| {
                if now.duration_since(*occ) <= per_ip_min_interval {
                    res = Err(occ.elapsed());
                } else {
                    // in list, expired
                    *occ = now;
                }
            })
            .or_insert(now);
        res
    }
}

/// To be called from a `thread::spawn` invocation
///
/// Runs the bootstrap management in a dedicated tokio runtime context. A failed bootstrap session
/// will fail silently locally, but will send a message to the client with an error message.
///
/// The arc_counter variable is used as a proxy to keep track the number of active bootstrap
/// sessions.
#[allow(clippy::too_many_arguments)]
fn run_bootstrap_session(
    mut server: BootstrapServerBinder,
    arc_counter: Arc<()>,
    config: BootstrapConfig,
    remote_addr: SocketAddr,
    data_execution: Arc<RwLock<FinalState>>,
    version: Version,
    consensus_command_sender: Box<dyn ConsensusController>,
    network_command_sender: NetworkCommandSender,
) {
    debug!("running bootstrap for peer {}", remote_addr);
    let session_context = Runtime::new().unwrap();
    session_context.handle().block_on(async move {
        let res = tokio::time::timeout(
            config.bootstrap_timeout.into(),
            manage_bootstrap(
                &config,
                &mut server,
                data_execution,
                version,
                consensus_command_sender,
                network_command_sender,
            ),
        )
        .await;
        // This drop allows the server to accept new connections before having to complete the error notifications
        // account for this session being finished, as well as the root-instance
        massa_trace!("bootstrap.session.finished", {
            "sessions_remaining": Arc::strong_count(&arc_counter) - 2
        });
        drop(arc_counter);
        match res {
            Ok(mgmt) => match mgmt {
                Ok(_) => {
                    info!("bootstrapped peer {}", remote_addr);
                }
                Err(BootstrapError::ReceivedError(error)) => debug!(
                    "bootstrap serving error received from peer {}: {}",
                    remote_addr, error
                ),
                Err(err) => {
                    debug!("bootstrap serving error for peer {}: {}", remote_addr, err);
                    // We allow unused result because we don't care if an error is thrown when
                    // sending the error message to the server we will close the socket anyway.
                    let _ = server.send_error(err.to_string()).await;
                }
            },
            Err(_timeout) => {
                debug!("bootstrap timeout for peer {}", remote_addr);
                // We allow unused result because we don't care if an error is thrown when
                // sending the error message to the server we will close the socket anyway.
                let _ = server
                    .send_error(format!(
                        "Bootstrap process timedout ({})",
                        format_duration(config.bootstrap_timeout.to_duration())
                    ))
                    .await;
            }
        }
    });
}

#[allow(clippy::too_many_arguments)]
pub async fn stream_bootstrap_information(
    server: &mut BootstrapServerBinder,
    final_state: Arc<RwLock<FinalState>>,
    consensus_controller: Box<dyn ConsensusController>,
    mut last_slot: Option<Slot>,
    mut last_ledger_step: StreamingStep<Vec<u8>>,
    mut last_pool_step: StreamingStep<AsyncMessageId>,
    mut last_cycle_step: StreamingStep<u64>,
    mut last_credits_step: StreamingStep<Slot>,
    mut last_ops_step: StreamingStep<Slot>,
    mut last_consensus_step: StreamingStep<PreHashSet<BlockId>>,
    write_timeout: Duration,
) -> Result<(), BootstrapError> {
    loop {
        #[cfg(test)]
        {
            // Necessary for test_bootstrap_server in tests/scenarios.rs
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        let current_slot;
        let ledger_part;
        let async_pool_part;
        let pos_cycle_part;
        let pos_credits_part;
        let exec_ops_part;
        let final_state_changes;

        let mut slot_too_old = false;

        // Scope of the final state read
        {
            let final_state_read = final_state.read();
            let (data, new_ledger_step) = final_state_read
                .ledger
                .get_ledger_part(last_ledger_step.clone())?;
            ledger_part = data;

            let (pool_data, new_pool_step) =
                final_state_read.async_pool.get_pool_part(last_pool_step);
            async_pool_part = pool_data;

            let (cycle_data, new_cycle_step) = final_state_read
                .pos_state
                .get_cycle_history_part(last_cycle_step)?;
            pos_cycle_part = cycle_data;

            let (credits_data, new_credits_step) = final_state_read
                .pos_state
                .get_deferred_credits_part(last_credits_step);
            pos_credits_part = credits_data;

            let (ops_data, new_ops_step) = final_state_read
                .executed_ops
                .get_executed_ops_part(last_ops_step);
            exec_ops_part = ops_data;

            if let Some(slot) = last_slot && slot != final_state_read.slot {
                if slot > final_state_read.slot {
                    return Err(BootstrapError::GeneralError(
                        "Bootstrap cursor set to future slot".to_string(),
                    ));
                }
                final_state_changes = match final_state_read.get_state_changes_part(
                    slot,
                    new_ledger_step.clone(),
                    new_pool_step,
                    new_cycle_step,
                    new_credits_step,
                    new_ops_step,
                ) {
                    Ok(data) => data,
                    Err(err) if matches!(err, FinalStateError::InvalidSlot(_)) => {
                        slot_too_old = true;
                        Vec::default()
                    }
                    Err(err) => return Err(BootstrapError::FinalStateError(err)),
                };
            } else {
                final_state_changes = Vec::new();
            }

            // Update cursors for next turn
            last_ledger_step = new_ledger_step;
            last_pool_step = new_pool_step;
            last_cycle_step = new_cycle_step;
            last_credits_step = new_credits_step;
            last_ops_step = new_ops_step;
            last_slot = Some(final_state_read.slot);
            current_slot = final_state_read.slot;
        }

        if slot_too_old {
            match server
                .send_msg(write_timeout, BootstrapServerMessage::SlotTooOld)
                .await
            {
                Err(_) => Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "SlotTooOld message send timed out",
                )
                .into()),
                Ok(Err(e)) => Err(e),
                Ok(Ok(_)) => Ok(()),
            }?;
            return Ok(());
        }

        // Setup final state global cursor
        let final_state_global_step = if last_ledger_step.finished()
            && last_pool_step.finished()
            && last_cycle_step.finished()
            && last_credits_step.finished()
            && last_ops_step.finished()
        {
            StreamingStep::Finished(Some(current_slot))
        } else {
            StreamingStep::Ongoing(current_slot)
        };

        // Setup final state changes cursor
        let final_state_changes_step = if final_state_changes.is_empty() {
            StreamingStep::Finished(Some(current_slot))
        } else {
            StreamingStep::Ongoing(current_slot)
        };

        // Stream consensus blocks if final state base bootstrap is finished
        let mut consensus_part = BootstrapableGraph {
            final_blocks: Default::default(),
        };
        let mut consensus_outdated_ids: PreHashSet<BlockId> = PreHashSet::default();
        if final_state_global_step.finished() {
            let (part, outdated_ids, new_consensus_step) = consensus_controller
                .get_bootstrap_part(last_consensus_step, final_state_changes_step)?;
            consensus_part = part;
            consensus_outdated_ids = outdated_ids;
            last_consensus_step = new_consensus_step;
        }

        // Logs for an easier diagnostic if needed
        debug!(
            "Final state bootstrap cursor: {:?}",
            final_state_global_step
        );
        debug!(
            "Consensus blocks bootstrap cursor: {:?}",
            last_consensus_step
        );
        if let StreamingStep::Ongoing(ids) = &last_consensus_step {
            debug!("Consensus bootstrap cursor length: {}", ids.len());
        }

        // If the consensus streaming is finished (also meaning that consensus slot == final state slot) exit
        if final_state_global_step.finished()
            && final_state_changes_step.finished()
            && last_consensus_step.finished()
        {
            match server
                .send_msg(write_timeout, BootstrapServerMessage::BootstrapFinished)
                .await
            {
                Err(_) => Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "bootstrap ask ledger part send timed out",
                )
                .into()),
                Ok(Err(e)) => Err(e),
                Ok(Ok(_)) => Ok(()),
            }?;
            break;
        }

        // At this point we know that consensus, final state or both are not finished
        match server
            .send_msg(
                write_timeout,
                BootstrapServerMessage::BootstrapPart {
                    slot: current_slot,
                    ledger_part,
                    async_pool_part,
                    pos_cycle_part,
                    pos_credits_part,
                    exec_ops_part,
                    final_state_changes,
                    consensus_part,
                    consensus_outdated_ids,
                },
            )
            .await
        {
            Err(_) => Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "bootstrap ask ledger part send timed out",
            )
            .into()),
            Ok(Err(e)) => Err(e),
            Ok(Ok(_)) => Ok(()),
        }?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn manage_bootstrap(
    bootstrap_config: &BootstrapConfig,
    server: &mut BootstrapServerBinder,
    final_state: Arc<RwLock<FinalState>>,
    version: Version,
    consensus_controller: Box<dyn ConsensusController>,
    network_command_sender: NetworkCommandSender,
) -> Result<(), BootstrapError> {
    massa_trace!("bootstrap.lib.manage_bootstrap", {});
    let read_error_timeout: std::time::Duration = bootstrap_config.read_error_timeout.into();

    match tokio::time::timeout(
        bootstrap_config.read_timeout.into(),
        server.handshake(version),
    )
    .await
    {
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "bootstrap handshake send timed out",
            )
            .into())
        }
        Ok(Err(e)) => return Err(e),
        Ok(Ok(_)) => (),
    };

    match tokio::time::timeout(read_error_timeout, server.next()).await {
        Err(_) => (),
        Ok(Err(e)) => return Err(e),
        Ok(Ok(BootstrapClientMessage::BootstrapError { error })) => {
            return Err(BootstrapError::GeneralError(error))
        }
        Ok(Ok(msg)) => return Err(BootstrapError::UnexpectedClientMessage(msg)),
    };

    let write_timeout: std::time::Duration = bootstrap_config.write_timeout.into();

    // Sync clocks.
    let server_time = MassaTime::now()?;

    match server
        .send_msg(
            write_timeout,
            BootstrapServerMessage::BootstrapTime {
                server_time,
                version,
            },
        )
        .await
    {
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "bootstrap clock send timed out",
        )
        .into()),
        Ok(Err(e)) => Err(e),
        Ok(Ok(_)) => Ok(()),
    }?;

    loop {
        match tokio::time::timeout(bootstrap_config.read_timeout.into(), server.next()).await {
            Err(_) => break Ok(()),
            Ok(Err(e)) => break Err(e),
            Ok(Ok(msg)) => match msg {
                BootstrapClientMessage::AskBootstrapPeers => {
                    match server
                        .send_msg(
                            write_timeout,
                            BootstrapServerMessage::BootstrapPeers {
                                peers: network_command_sender.get_bootstrap_peers().await?,
                            },
                        )
                        .await
                    {
                        Err(_) => Err(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "bootstrap peers send timed out",
                        )
                        .into()),
                        Ok(Err(e)) => Err(e),
                        Ok(Ok(_)) => Ok(()),
                    }?;
                }
                BootstrapClientMessage::AskBootstrapPart {
                    last_slot,
                    last_ledger_step,
                    last_pool_step,
                    last_cycle_step,
                    last_credits_step,
                    last_ops_step,
                    last_consensus_step,
                } => {
                    stream_bootstrap_information(
                        server,
                        final_state.clone(),
                        consensus_controller.clone(),
                        last_slot,
                        last_ledger_step,
                        last_pool_step,
                        last_cycle_step,
                        last_credits_step,
                        last_ops_step,
                        last_consensus_step,
                        write_timeout,
                    )
                    .await?;
                }
                BootstrapClientMessage::BootstrapSuccess => break Ok(()),
                BootstrapClientMessage::BootstrapError { error } => {
                    break Err(BootstrapError::ReceivedError(error));
                }
            },
        };
    }
}

// Stable means of providing compiler optimisation hints
// Also provides a self-documenting tool to communicate likely/unlikely code-paths
// https://users.rust-lang.org/t/compiler-hint-for-unlikely-likely-for-if-branches/62102/4
#[inline]
#[cold]
fn cold() {}

#[inline]
fn _likely(b: bool) -> bool {
    if !b {
        cold()
    }
    b
}

#[inline]
fn unlikely(b: bool) -> bool {
    if b {
        cold()
    }
    b
}
