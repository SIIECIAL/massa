// Copyright (c) 2022 MASSA LABS <info@massa.net>

use super::tools::{
    get_boot_state, get_peers, get_random_final_state_bootstrap, get_random_ledger_changes,
};
use crate::tests::tools::{
    get_random_async_pool_changes, get_random_executed_ops_changes, get_random_pos_changes,
};
use crate::BootstrapConfig;
use crate::{
    establisher::{MockBSConnector, MockBSListener},
    get_state, start_bootstrap_server,
    tests::tools::{assert_eq_bootstrap_graph, get_bootstrap_config},
};
use massa_async_pool::AsyncPoolConfig;
use massa_consensus_exports::{
    bootstrapable_graph::BootstrapableGraph, test_exports::MockConsensusControllerImpl,
};
use massa_executed_ops::ExecutedOpsConfig;
use massa_final_state::{
    test_exports::{assert_eq_final_state, assert_eq_final_state_hash},
    FinalState, FinalStateConfig, StateChanges,
};
use massa_hash::{Hash, HASH_SIZE_BYTES};
use massa_ledger_exports::LedgerConfig;
use massa_models::config::{MIP_STORE_STATS_BLOCK_CONSIDERED, MIP_STORE_STATS_COUNTERS_MAX};
use massa_models::{
    address::Address, config::MAX_DATASTORE_VALUE_LENGTH, node::NodeId, slot::Slot,
    streaming_step::StreamingStep, version::Version,
};
use massa_models::{
    config::{
        MAX_ASYNC_MESSAGE_DATA, MAX_ASYNC_POOL_LENGTH, MAX_DATASTORE_KEY_LENGTH, POS_SAVED_CYCLES,
    },
    prehash::PreHashSet,
};
use massa_network_exports::MockNetworkCommandSender;
use massa_pos_exports::{
    test_exports::assert_eq_pos_selection, PoSConfig, PoSFinalState, SelectorConfig,
};
use massa_pos_worker::start_selector_worker;
use massa_signature::KeyPair;
use massa_time::MassaTime;
use massa_versioning_worker::versioning::{
    MipComponent, MipInfo, MipState, MipStatsConfig, MipStore,
};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::{path::PathBuf, str::FromStr, sync::Arc, time::Duration};
use tempfile::TempDir;

lazy_static::lazy_static! {
    pub static ref BOOTSTRAP_CONFIG_KEYPAIR: (BootstrapConfig, KeyPair) = {
        let keypair = KeyPair::generate();
        (get_bootstrap_config(NodeId::new(keypair.get_public_key())), keypair)
    };
}

#[test]
fn test_bootstrap_server() {
    let thread_count = 2;
    let periods_per_cycle = 2;
    let (bootstrap_config, keypair): &(BootstrapConfig, KeyPair) = &BOOTSTRAP_CONFIG_KEYPAIR;
    let rolls_path = PathBuf::from_str("../massa-node/base_config/initial_rolls.json").unwrap();
    let genesis_address = Address::from_public_key(&KeyPair::generate().get_public_key());

    // let (consensus_controller, mut consensus_event_receiver) =
    //     MockConsensusController::new_with_receiver();
    // let (network_cmd_tx, mut network_cmd_rx) = mpsc::channel::<NetworkCommand>(5);

    // create a MIP store
    let mip_stats_cfg = MipStatsConfig {
        block_count_considered: MIP_STORE_STATS_BLOCK_CONSIDERED,
        counters_max: MIP_STORE_STATS_COUNTERS_MAX,
    };
    let mi_1 = MipInfo {
        name: "MIP-0002".to_string(),
        version: 2,
        components: HashMap::from([(MipComponent::Address, 1)]),
        start: MassaTime::from(5),
        timeout: MassaTime::from(10),
        activation_delay: MassaTime::from(4),
    };
    let state_1 = MipState::new(MassaTime::from(3));
    let mip_store = MipStore::try_from(([(mi_1, state_1)], mip_stats_cfg.clone())).unwrap();

    // setup final state local config
    let temp_dir = TempDir::new().unwrap();
    let final_state_local_config = FinalStateConfig {
        ledger_config: LedgerConfig {
            thread_count,
            initial_ledger_path: "".into(),
            disk_ledger_path: temp_dir.path().to_path_buf(),
            max_key_length: MAX_DATASTORE_KEY_LENGTH,
            max_ledger_part_size: 100_000,
            max_datastore_value_length: MAX_DATASTORE_VALUE_LENGTH,
        },
        async_pool_config: AsyncPoolConfig {
            thread_count,
            max_length: MAX_ASYNC_POOL_LENGTH,
            max_async_message_data: MAX_ASYNC_MESSAGE_DATA,
            bootstrap_part_size: 100,
        },
        pos_config: PoSConfig {
            periods_per_cycle,
            thread_count,
            cycle_history_length: POS_SAVED_CYCLES,
            credits_bootstrap_part_size: 100,
        },
        executed_ops_config: ExecutedOpsConfig {
            thread_count,
            bootstrap_part_size: 10,
        },
        final_history_length: 100,
        initial_seed_string: "".into(),
        initial_rolls_path: "".into(),
        thread_count,
        periods_per_cycle,
    };

    // setup selector local config
    let selector_local_config = SelectorConfig {
        thread_count,
        periods_per_cycle,
        genesis_address,
        ..Default::default()
    };

    // start proof-of-stake selectors
    let (mut server_selector_manager, server_selector_controller) =
        start_selector_worker(selector_local_config.clone())
            .expect("could not start server selector controller");
    let (mut client_selector_manager, client_selector_controller) =
        start_selector_worker(selector_local_config)
            .expect("could not start client selector controller");

    // setup final states
    let final_state_server = Arc::new(RwLock::new(get_random_final_state_bootstrap(
        PoSFinalState::new(
            final_state_local_config.pos_config.clone(),
            "",
            &rolls_path,
            server_selector_controller.clone(),
            Hash::from_bytes(&[0; HASH_SIZE_BYTES]),
        )
        .unwrap(),
        final_state_local_config.clone(),
    )));
    let final_state_client = Arc::new(RwLock::new(FinalState::create_final_state(
        PoSFinalState::new(
            final_state_local_config.pos_config.clone(),
            "",
            &rolls_path,
            client_selector_controller.clone(),
            Hash::from_bytes(&[0; HASH_SIZE_BYTES]),
        )
        .unwrap(),
        final_state_local_config,
    )));
    let final_state_client_clone = final_state_client.clone();
    let final_state_server_clone1 = final_state_server.clone();
    let final_state_server_clone2 = final_state_server.clone();

    let (mock_bs_listener, mock_remote_connector) = conn_establishment_mocks();

    // Setup network command mock-story: hard-code the result of getting bootstrap peers
    let mut mocked1 = MockNetworkCommandSender::new();
    let mut mocked2 = MockNetworkCommandSender::new();
    mocked2
        .expect_get_bootstrap_peers()
        .times(1)
        .returning(|| Ok(get_peers()));

    mocked1.expect_clone().return_once(move || mocked2);
    let mut stream_mock1 = Box::new(MockConsensusControllerImpl::new());
    let mut stream_mock2 = Box::new(MockConsensusControllerImpl::new());
    let mut stream_mock3 = Box::new(MockConsensusControllerImpl::new());
    let mut seq = mockall::Sequence::new();

    let sent_graph = get_boot_state();
    let sent_graph_clone = sent_graph.clone();
    stream_mock3
        .expect_get_bootstrap_part()
        .times(10)
        .in_sequence(&mut seq)
        .returning(move |_, slot| {
            if StreamingStep::Ongoing(Slot::new(1, 1)) == slot {
                Ok((
                    sent_graph_clone.clone(),
                    PreHashSet::default(),
                    StreamingStep::Started,
                ))
            } else {
                Ok((
                    BootstrapableGraph {
                        final_blocks: vec![],
                    },
                    PreHashSet::default(),
                    StreamingStep::Finished(None),
                ))
            }
        });
    stream_mock2
        .expect_clone_box()
        .return_once(move || stream_mock3);
    stream_mock1
        .expect_clone_box()
        .return_once(move || stream_mock2);

    let cloned_store = mip_store.clone();
    let bootstrap_manager_thread = std::thread::Builder::new()
        .name("bootstrap_thread".to_string())
        .spawn(move || {
            start_bootstrap_server::<MockNetworkCommandSender>(
                bootstrap_config.listen_addr.unwrap(),
                stream_mock1,
                mocked1,
                final_state_server_clone1,
                bootstrap_config.clone(),
                keypair.clone(),
                Version::from_str("TEST.1.10").unwrap(),
                cloned_store,
            )
            .unwrap()
        })
        .unwrap();

    // launch the modifier thread
    let list_changes: Arc<RwLock<Vec<(Slot, StateChanges)>>> = Arc::new(RwLock::new(Vec::new()));
    let list_changes_clone = list_changes.clone();
    let mod_thread = std::thread::Builder::new()
        .name("modifier thread".to_string())
        .spawn(move || {
            for _ in 0..10 {
                std::thread::sleep(Duration::from_millis(500));
                let mut final_write = final_state_server_clone2.write();
                let next = final_write.slot.get_next_slot(thread_count).unwrap();
                final_write.slot = next;
                let changes = StateChanges {
                    pos_changes: get_random_pos_changes(10),
                    ledger_changes: get_random_ledger_changes(10),
                    async_pool_changes: get_random_async_pool_changes(10),
                    executed_ops_changes: get_random_executed_ops_changes(10),
                };
                final_write
                    .changes_history
                    .push_back((next, changes.clone()));
                let mut list_changes_write = list_changes_clone.write();
                list_changes_write.push((next, changes));
            }
        })
        .unwrap();

    // mock_remote_connector
    //     .expect_connect_timeout()
    //     .times(1)
    //     .returning(|_, _| todo!());
    // launch the get_state process
    let bootstrap_res = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(get_state(
            bootstrap_config,
            final_state_client_clone,
            mock_remote_connector,
            Version::from_str("TEST.1.10").unwrap(),
            MassaTime::now().unwrap().saturating_sub(1000.into()),
            None,
            None,
        ))
        .unwrap();

    // apply the changes to the server state before matching with the client
    {
        let mut final_state_server_write = final_state_server.write();
        let list_changes_read = list_changes.read().clone();
        // note: skip the first change to match the update loop behaviour
        for (slot, change) in list_changes_read.iter().skip(1) {
            final_state_server_write
                .pos_state
                .apply_changes(change.pos_changes.clone(), *slot, false)
                .unwrap();
            final_state_server_write.ledger.apply_changes(
                change.ledger_changes.clone(),
                *slot,
                None,
            );
            final_state_server_write
                .async_pool
                .apply_changes_unchecked(&change.async_pool_changes);
            final_state_server_write
                .executed_ops
                .apply_changes(change.executed_ops_changes.clone(), *slot);
        }
    }
    // Make sure the modifier thread has done its job
    mod_thread.join().unwrap();

    // check final states
    assert_eq_final_state(&final_state_server.read(), &final_state_client.read());
    assert_eq_final_state_hash(&final_state_server.read(), &final_state_client.read());

    // compute initial draws
    final_state_server.write().compute_initial_draws().unwrap();
    final_state_client.write().compute_initial_draws().unwrap();

    // check selection draw
    let server_selection = server_selector_controller.get_entire_selection();
    let client_selection = client_selector_controller.get_entire_selection();
    assert_eq_pos_selection(&server_selection, &client_selection);

    // check peers
    assert_eq!(
        get_peers().0,
        bootstrap_res.peers.unwrap().0,
        "mismatch between sent and received peers"
    );

    // check graphs
    assert_eq_bootstrap_graph(&sent_graph, &bootstrap_res.graph.unwrap());

    // check mip store
    let mip_raw_orig = mip_store.0.read().to_owned();
    let mip_raw_received = bootstrap_res.mip_store.unwrap().0.read().to_owned();
    assert_eq!(mip_raw_orig, mip_raw_received);

    // stop bootstrap server
    bootstrap_manager_thread
        .join()
        .unwrap()
        .stop()
        .expect("could not stop bootstrap server");

    // stop selector controllers
    server_selector_manager.stop();
    client_selector_manager.stop();
}

fn conn_establishment_mocks() -> (MockBSListener, MockBSConnector) {
    // Setup the server/client connection
    // Bind a TcpListener to localhost on a specific port
    let listener = std::net::TcpListener::bind("127.0.0.1:8069").unwrap();

    // Due to the limitations of the mocking system, the listener must loop-accept in a dedicated
    // thread. We use a channel to make the connection available in the mocked `accept` method
    let (conn_tx, conn_rx) = std::sync::mpsc::sync_channel(100);
    let conn = std::thread::Builder::new()
        .name("mock conn connect".to_string())
        .spawn(|| std::net::TcpStream::connect("127.0.0.1:8069").unwrap())
        .unwrap();
    std::thread::Builder::new()
        .name("mock-listen-loop".to_string())
        .spawn(move || loop {
            conn_tx.send(listener.accept().unwrap()).unwrap()
        })
        .unwrap();

    // Mock the connection setups
    // TODO: Why is it twice, and not just once?
    let mut mock_bs_listener = MockBSListener::new();
    mock_bs_listener
        .expect_accept()
        .times(2)
        // Mock the `accept` method here by receiving from the listen-loop thread
        .returning(move || Ok(conn_rx.recv().unwrap()));

    let mut mock_remote_connector = MockBSConnector::new();
    mock_remote_connector
        .expect_connect_timeout()
        .times(1)
        .return_once(move |_, _| Ok(conn.join().unwrap()));
    (mock_bs_listener, mock_remote_connector)
}
