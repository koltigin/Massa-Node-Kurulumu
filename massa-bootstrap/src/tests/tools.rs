// Copyright (c) 2022 MASSA LABS <info@massa.net>

use super::mock_establisher::Duplex;
use crate::settings::BootstrapConfig;
use bitvec::vec::BitVec;
use massa_async_pool::test_exports::{create_async_pool, get_random_message};
use massa_async_pool::{AsyncPoolChanges, Change};
use massa_consensus_exports::commands::ConsensusCommand;
use massa_final_state::test_exports::create_final_state;
use massa_final_state::{ExecutedOps, FinalState};
use massa_graph::export_active_block::ExportActiveBlockSerializer;
use massa_graph::{export_active_block::ExportActiveBlock, BootstrapableGraph};
use massa_graph::{BootstrapableGraphDeserializer, BootstrapableGraphSerializer};
use massa_hash::Hash;
use massa_ledger_exports::{LedgerChanges, LedgerEntry, SetUpdateOrDelete};
use massa_ledger_worker::test_exports::create_final_ledger;
use massa_models::config::{
    BOOTSTRAP_RANDOMNESS_SIZE_BYTES, ENDORSEMENT_COUNT, MAX_ADVERTISE_LENGTH,
    MAX_BOOTSTRAP_ASYNC_POOL_CHANGES, MAX_BOOTSTRAP_BLOCKS, MAX_BOOTSTRAP_ERROR_LENGTH,
    MAX_BOOTSTRAP_FINAL_STATE_PARTS_SIZE, MAX_BOOTSTRAP_MESSAGE_SIZE, MAX_DATASTORE_ENTRY_COUNT,
    MAX_DATASTORE_KEY_LENGTH, MAX_DATASTORE_VALUE_LENGTH, MAX_DATA_ASYNC_MESSAGE,
    MAX_FUNCTION_NAME_LENGTH, MAX_LEDGER_CHANGES_COUNT, MAX_OPERATIONS_PER_BLOCK,
    MAX_PARAMETERS_SIZE, PERIODS_PER_CYCLE, THREAD_COUNT,
};
use massa_models::{
    address::Address,
    amount::Amount,
    block::BlockSerializer,
    block::{Block, BlockHeader, BlockHeaderSerializer, BlockId},
    endorsement::Endorsement,
    endorsement::EndorsementSerializer,
    operation::OperationId,
    prehash::PreHashMap,
    slot::Slot,
    wrapped::Id,
    wrapped::WrappedContent,
};
use massa_network_exports::{BootstrapPeers, NetworkCommand};
use massa_pos_exports::{CycleInfo, DeferredCredits, PoSChanges, PoSFinalState, ProductionStats};
use massa_serialization::{DeserializeError, Deserializer, Serializer};
use massa_signature::{KeyPair, PublicKey, Signature};
use massa_time::MassaTime;
use rand::Rng;
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;
use std::{
    collections::BTreeMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::{sync::mpsc::Receiver, time::sleep};

pub const BASE_BOOTSTRAP_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(169, 202, 0, 10));

/// generates a small random number of bytes
fn get_some_random_bytes() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0usize..rng.gen_range(1..10))
        .map(|_| rand::random::<u8>())
        .collect()
}

/// generates a random ledger entry
fn get_random_ledger_entry() -> LedgerEntry {
    let mut rng = rand::thread_rng();
    let parallel_balance = Amount::from_raw(rng.gen::<u64>());
    let sequential_balance = Amount::from_raw(rng.gen::<u64>());
    let bytecode: Vec<u8> = get_some_random_bytes();
    let mut datastore = BTreeMap::new();
    for _ in 0usize..rng.gen_range(0..10) {
        let key = get_some_random_bytes();
        let value = get_some_random_bytes();
        datastore.insert(key, value);
    }
    LedgerEntry {
        sequential_balance,
        parallel_balance,
        bytecode,
        datastore,
    }
}

pub fn get_random_ledger_changes(r_limit: u64) -> LedgerChanges {
    let mut changes = LedgerChanges::default();
    for _ in 0..r_limit {
        changes.0.insert(
            get_random_address(),
            SetUpdateOrDelete::Set(LedgerEntry {
                sequential_balance: Amount::from_raw(r_limit),
                parallel_balance: Amount::from_raw(r_limit),
                bytecode: Vec::default(),
                datastore: BTreeMap::default(),
            }),
        );
    }
    changes
}

/// generates random PoS cycles info
fn get_random_pos_cycles_info(
    r_limit: u64,
    opt_seed: bool,
) -> (
    BTreeMap<Address, u64>,
    PreHashMap<Address, ProductionStats>,
    BitVec<u8>,
) {
    let mut rng = rand::thread_rng();
    let mut roll_counts = BTreeMap::default();
    let mut production_stats = PreHashMap::default();
    let mut rng_seed: BitVec<u8> = BitVec::default();

    for i in 0u64..r_limit {
        roll_counts.insert(get_random_address(), i);
        production_stats.insert(
            get_random_address(),
            ProductionStats {
                block_success_count: i * 3,
                block_failure_count: i,
            },
        );
    }
    // note: extra seed is used in the changes test to compensate for the update loop skipping the first change
    if opt_seed {
        rng_seed.push(rng.gen_range(0..2) == 1);
    }
    rng_seed.push(rng.gen_range(0..2) == 1);
    (roll_counts, production_stats, rng_seed)
}

/// generates random PoS deferred credits
fn get_random_deferred_credits(r_limit: u64) -> DeferredCredits {
    let mut deferred_credits = DeferredCredits::default();

    for i in 0u64..r_limit {
        let mut credits = PreHashMap::default();
        for j in 0u64..r_limit {
            credits.insert(get_random_address(), Amount::from_raw(j));
        }
        deferred_credits.0.insert(
            Slot {
                period: i,
                thread: 0,
            },
            credits,
        );
    }
    deferred_credits
}

/// generates a random PoS final state
fn get_random_pos_state(r_limit: u64, pos: PoSFinalState) -> PoSFinalState {
    let mut cycle_history = VecDeque::new();
    let (roll_counts, production_stats, rng_seed) = get_random_pos_cycles_info(r_limit, true);
    cycle_history.push_back(CycleInfo {
        cycle: 0,
        roll_counts,
        complete: false,
        rng_seed,
        production_stats,
    });
    let deferred_credits = get_random_deferred_credits(r_limit);
    PoSFinalState {
        cycle_history,
        deferred_credits,
        ..pos
    }
}

/// generates random PoS changes
pub fn get_random_pos_changes(r_limit: u64) -> PoSChanges {
    let deferred_credits = get_random_deferred_credits(r_limit);
    let (roll_counts, production_stats, seed_bits) = get_random_pos_cycles_info(r_limit, false);
    PoSChanges {
        seed_bits,
        roll_changes: roll_counts.into_iter().collect(),
        production_stats,
        deferred_credits,
    }
}

pub fn get_random_async_pool_changes(r_limit: u64) -> AsyncPoolChanges {
    let mut changes = AsyncPoolChanges::default();
    for _ in 0..r_limit {
        let mut message = get_random_message();
        message.gas_price = Amount::from_str("1_000_000").unwrap();
        changes.0.push(Change::Add(message.compute_id(), message));
    }
    changes
}

pub fn get_random_executed_ops(r_limit: u64) -> ExecutedOps {
    let mut ops = ExecutedOps::default();
    for _ in 0..r_limit {
        ops.insert(
            OperationId::new(Hash::compute_from(&get_some_random_bytes())),
            Slot {
                period: 500,
                thread: 0,
            },
        );
    }
    ops
}

/// generates a random bootstrap state for the final state
pub fn get_random_final_state_bootstrap(pos: PoSFinalState) -> FinalState {
    let r_limit: u64 = 50;

    let mut sorted_ledger = HashMap::new();
    let mut messages = BTreeMap::new();
    for _ in 0..r_limit {
        let message = get_random_message();
        messages.insert(message.compute_id(), message);
    }
    for _ in 0..r_limit {
        sorted_ledger.insert(get_random_address(), get_random_ledger_entry());
    }
    // insert the last possible address to prevent the last cursor to move when testing the changes
    sorted_ledger.insert(Address::from_bytes(&[255; 32]), get_random_ledger_entry());

    let slot = Slot::new(0, 0);
    let final_ledger = create_final_ledger(Some(sorted_ledger), Default::default());
    let async_pool = create_async_pool(Default::default(), messages);

    create_final_state(
        Default::default(),
        slot,
        Box::new(final_ledger),
        async_pool,
        VecDeque::new(),
        get_random_pos_state(r_limit, pos),
        get_random_executed_ops(r_limit),
    )
}

pub fn get_dummy_block_id(s: &str) -> BlockId {
    BlockId(Hash::compute_from(s.as_bytes()))
}

pub fn get_random_public_key() -> PublicKey {
    let priv_key = KeyPair::generate();
    priv_key.get_public_key()
}

pub fn get_random_address() -> Address {
    let priv_key = KeyPair::generate();
    Address::from_public_key(&priv_key.get_public_key())
}

pub fn get_dummy_signature(s: &str) -> Signature {
    let priv_key = KeyPair::generate();
    priv_key.sign(&Hash::compute_from(s.as_bytes())).unwrap()
}

pub fn get_bootstrap_config(bootstrap_public_key: PublicKey) -> BootstrapConfig {
    BootstrapConfig {
        bind: Some("0.0.0.0:31244".parse().unwrap()),
        connect_timeout: 200.into(),
        retry_delay: 200.into(),
        max_ping: MassaTime::from_millis(500),
        read_timeout: 1000.into(),
        write_timeout: 1000.into(),
        read_error_timeout: 200.into(),
        write_error_timeout: 200.into(),
        bootstrap_list: vec![(SocketAddr::new(BASE_BOOTSTRAP_IP, 16), bootstrap_public_key)],
        enable_clock_synchronization: true,
        cache_duration: 10000.into(),
        max_simultaneous_bootstraps: 2,
        ip_list_max_size: 10,
        per_ip_min_interval: 10000.into(),
        max_bytes_read_write: std::f64::INFINITY,
        max_bootstrap_message_size: MAX_BOOTSTRAP_MESSAGE_SIZE,
        max_datastore_key_length: MAX_DATASTORE_KEY_LENGTH,
        randomness_size_bytes: BOOTSTRAP_RANDOMNESS_SIZE_BYTES,
        thread_count: THREAD_COUNT,
        periods_per_cycle: PERIODS_PER_CYCLE,
        endorsement_count: ENDORSEMENT_COUNT,
        max_advertise_length: MAX_ADVERTISE_LENGTH,
        max_bootstrap_async_pool_changes: MAX_BOOTSTRAP_ASYNC_POOL_CHANGES,
        max_bootstrap_blocks_length: MAX_BOOTSTRAP_BLOCKS,
        max_bootstrap_error_length: MAX_BOOTSTRAP_ERROR_LENGTH,
        max_bootstrap_final_state_parts_size: MAX_BOOTSTRAP_FINAL_STATE_PARTS_SIZE,
        max_data_async_message: MAX_DATA_ASYNC_MESSAGE,
        max_operations_per_blocks: MAX_OPERATIONS_PER_BLOCK,
        max_datastore_entry_count: MAX_DATASTORE_ENTRY_COUNT,
        max_datastore_value_length: MAX_DATASTORE_VALUE_LENGTH,
        max_function_name_length: MAX_FUNCTION_NAME_LENGTH,
        max_ledger_changes_count: MAX_LEDGER_CHANGES_COUNT,
        max_parameters_size: MAX_PARAMETERS_SIZE,
        max_changes_slot_count: 1000,
    }
}

pub async fn wait_consensus_command<F, T>(
    consensus_command_receiver: &mut Receiver<ConsensusCommand>,
    timeout: MassaTime,
    filter_map: F,
) -> Option<T>
where
    F: Fn(ConsensusCommand) -> Option<T>,
{
    let timer = sleep(timeout.into());
    tokio::pin!(timer);
    loop {
        tokio::select! {
            cmd = consensus_command_receiver.recv() => match cmd {
                Some(orig_evt) => if let Some(res_evt) = filter_map(orig_evt) { return Some(res_evt); },
                _ => panic!("network event channel died")
            },
            _ = &mut timer => return None
        }
    }
}

pub async fn wait_network_command<F, T>(
    network_command_receiver: &mut Receiver<NetworkCommand>,
    timeout: MassaTime,
    filter_map: F,
) -> Option<T>
where
    F: Fn(NetworkCommand) -> Option<T>,
{
    let timer = sleep(timeout.into());
    tokio::pin!(timer);
    loop {
        tokio::select! {
            cmd = network_command_receiver.recv() => match cmd {
                Some(orig_evt) => if let Some(res_evt) = filter_map(orig_evt) { return Some(res_evt); },
                _ => panic!("network event channel died")
            },
            _ = &mut timer => return None
        }
    }
}

/// asserts that two `BootstrapableGraph` are equal
pub fn assert_eq_bootstrap_graph(v1: &BootstrapableGraph, v2: &BootstrapableGraph) {
    assert_eq!(
        v1.final_blocks.len(),
        v2.final_blocks.len(),
        "length mismatch"
    );
    let serializer = ExportActiveBlockSerializer::new();
    let mut data1: Vec<u8> = Vec::new();
    let mut data2: Vec<u8> = Vec::new();
    for (item1, item2) in v1.final_blocks.iter().zip(v2.final_blocks.iter()) {
        serializer.serialize(item1, &mut data1).unwrap();
        serializer.serialize(item2, &mut data2).unwrap();
    }
    assert_eq!(data1, data2, "BootstrapableGraph mismatch")
}

pub fn get_boot_state() -> BootstrapableGraph {
    let keypair = KeyPair::generate();

    let block = Block::new_wrapped(
        Block {
            header: BlockHeader::new_wrapped(
                BlockHeader {
                    slot: Slot::new(1, 1),
                    parents: vec![get_dummy_block_id("p1"); THREAD_COUNT as usize],
                    operation_merkle_root: Hash::compute_from("op_hash".as_bytes()),
                    endorsements: vec![
                        Endorsement::new_wrapped(
                            Endorsement {
                                slot: Slot::new(1, 0),
                                index: 1,
                                endorsed_block: get_dummy_block_id("p1"),
                            },
                            EndorsementSerializer::new(),
                            &keypair,
                        )
                        .unwrap(),
                        Endorsement::new_wrapped(
                            Endorsement {
                                slot: Slot::new(4, 1),
                                index: 3,
                                endorsed_block: get_dummy_block_id("p1"),
                            },
                            EndorsementSerializer::new(),
                            &keypair,
                        )
                        .unwrap(),
                    ],
                },
                BlockHeaderSerializer::new(),
                &keypair,
            )
            .unwrap(),
            operations: Default::default(),
        },
        BlockSerializer::new(),
        &keypair,
    )
    .unwrap();

    // TODO: We currently lost information. Need to use shared storage
    let block1 = ExportActiveBlock {
        block,
        parents: vec![(get_dummy_block_id("b1"), 4777); THREAD_COUNT as usize],
        is_final: true,
        operations: Default::default(),
    };

    let boot_graph = BootstrapableGraph {
        final_blocks: vec![block1],
    };

    let bootstrapable_graph_serializer = BootstrapableGraphSerializer::new();
    let bootstrapable_graph_deserializer = BootstrapableGraphDeserializer::new(
        THREAD_COUNT,
        ENDORSEMENT_COUNT,
        MAX_BOOTSTRAP_BLOCKS,
        MAX_DATASTORE_VALUE_LENGTH,
        MAX_FUNCTION_NAME_LENGTH,
        MAX_PARAMETERS_SIZE,
        MAX_OPERATIONS_PER_BLOCK,
    );

    let mut bootstrapable_graph_serialized = Vec::new();
    bootstrapable_graph_serializer
        .serialize(&boot_graph, &mut bootstrapable_graph_serialized)
        .unwrap();
    let (_, bootstrapable_graph_deserialized) = bootstrapable_graph_deserializer
        .deserialize::<DeserializeError>(&bootstrapable_graph_serialized)
        .unwrap();

    assert_eq_bootstrap_graph(&bootstrapable_graph_deserialized, &boot_graph);

    boot_graph
}

pub fn get_peers() -> BootstrapPeers {
    BootstrapPeers(vec![
        "82.245.123.77".parse().unwrap(),
        "82.220.123.78".parse().unwrap(),
    ])
}

pub async fn bridge_mock_streams(mut side1: Duplex, mut side2: Duplex) {
    let mut buf1 = vec![0u8; 1024];
    let mut buf2 = vec![0u8; 1024];
    loop {
        tokio::select! {
            res1 = side1.read(&mut buf1) => match res1 {
                Ok(n1) => {
                    if n1 == 0 {
                        return;
                    }
                    if side2.write_all(&buf1[..n1]).await.is_err() {
                        return;
                    }
                },
                Err(_err) => {
                    return;
                }
            },
            res2 = side2.read(&mut buf2) => match res2 {
                Ok(n2) => {
                    if n2 == 0 {
                        return;
                    }
                    if side1.write_all(&buf2[..n2]).await.is_err() {
                        return;
                    }
                },
                Err(_err) => {
                    return;
                }
            },
        }
    }
}
