/// OPC UA integration tests — start a real server, connect a real client.
///
/// Pattern from async-opcua samples:
///   /private/tmp/async-opcua/samples/simple-client/src/main.rs
///   /private/tmp/async-opcua/async-opcua-0.18.0/tests/utils/tester.rs
///
/// Key points:
/// - `ServerBuilder::new_anonymous` + `server.run_with(listener)` for random port
/// - `ClientBuilder::new().trust_server_certs(true)` for no-cert test setup
/// - `session.wait_for_connection().await` before any reads
/// - Read values as `Variant::String(s)` → `s.value().as_deref()`
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use opcua::{
    client::{ClientBuilder, DataChangeCallback, IdentityToken, MonitoredItem},
    crypto::SecurityPolicy,
    types::{
        AttributeId, DataValue, MessageSecurityMode, MonitoredItemCreateRequest, NodeId,
        ReadValueId, TimestampsToReturn, UserTokenPolicy, Variant,
    },
};
use tokio::net::TcpListener;
use tokio::time::sleep;

use opcua_howick::config::{
    Config, HttpConfig, MachineConfig, OpcUaConfig, PlatTrunkConfig, SensorConfig,
};
use opcua_howick::job_server::opcua_server::run_server_with;
use opcua_howick::machine::{new_shared_state, MachineStatus};

// ── Counter for unique test PKI dirs ───────────────────────────────────────────

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn test_id() -> usize {
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

// ── Test helpers ───────────────────────────────────────────────────────────────

fn make_config(port: u16) -> Config {
    Config {
        opcua: OpcUaConfig {
            host: "127.0.0.1".into(),
            port,
            application_name: "howick-test".into(),
            namespace_uri: "urn:howick-frama".into(),
        },
        machine: MachineConfig {
            machine_name: "Test FRAMA".into(),
            job_input_dir: std::env::temp_dir().join("howick-opcua-test-input"),
            machine_input_dir: std::env::temp_dir().join("howick-opcua-test-machine"),
            machine_output_dir: std::env::temp_dir().join("howick-opcua-test-output"),
            usb_gadget_mode: false,
        },
        http: HttpConfig {
            host: "127.0.0.1".into(),
            port: 0,
        },
        plat_trunk: PlatTrunkConfig {
            url: "http://localhost:3000".into(),
            api_key: String::new(),
            status_push_interval_secs: 5,
        },
        sensor: SensorConfig::default(),
    }
}

/// Start OPC UA server on a random port. Returns the bound address.
async fn start_opcua_server() -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let config = make_config(addr.port());
    let state = new_shared_state();
    {
        state.write().await.status = MachineStatus::Idle;
    }
    let state_clone = state.clone();
    tokio::spawn(async move {
        run_server_with(listener, &config, state_clone).await.ok();
    });
    // Give the server a moment to start
    sleep(Duration::from_millis(300)).await;
    addr
}

/// Connect an OPC UA client to the test server. Returns session.
async fn connect_client(
    addr: std::net::SocketAddr,
    id: usize,
) -> std::sync::Arc<opcua::client::Session> {
    let url = format!("opc.tcp://127.0.0.1:{}/", addr.port());

    let mut client = ClientBuilder::new()
        .application_name("howick-test-client")
        .application_uri("urn:howick-test-client")
        .pki_dir(format!("target/tmp/pki-test-client-{id}"))
        .create_sample_keypair(true)
        .trust_server_certs(true)
        .session_retry_limit(1)
        .client()
        .unwrap();

    let (session, event_loop) = client
        .connect_to_matching_endpoint(
            (
                url.as_str(),
                SecurityPolicy::None.to_str(),
                MessageSecurityMode::None,
                UserTokenPolicy::anonymous(),
            ),
            IdentityToken::Anonymous,
        )
        .await
        .unwrap();

    event_loop.spawn();
    tokio::time::timeout(Duration::from_secs(5), session.wait_for_connection())
        .await
        .expect("OPC UA connection timeout");

    session
}

// ── Tests ──────────────────────────────────────────────────────────────────────

/// Connect to server and read Machine/Status — verifies the OPC UA address space is live.
#[tokio::test]
async fn opcua_read_machine_status() {
    let addr = start_opcua_server().await;
    let session = connect_client(addr, test_id()).await;

    // Resolve our namespace index
    let ns = get_namespace_index(&session, "urn:howick-frama")
        .await
        .unwrap_or(2);

    let results = session
        .read(
            &[ReadValueId {
                node_id: NodeId::new(ns, "Machine/Status"),
                attribute_id: AttributeId::Value as u32,
                ..Default::default()
            }],
            TimestampsToReturn::Both,
            0.0,
        )
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    // Status starts as "Idle" (set in start_opcua_server)
    // sync task pushes it within 500ms — wait for first sync
    sleep(Duration::from_millis(600)).await;

    let results = session
        .read(
            &[ReadValueId {
                node_id: NodeId::new(ns, "Machine/Status"),
                attribute_id: AttributeId::Value as u32,
                ..Default::default()
            }],
            TimestampsToReturn::Both,
            0.0,
        )
        .await
        .unwrap();

    let status = extract_string(&results[0]).unwrap_or_default();
    assert_eq!(status, "Idle", "Machine/Status should be Idle");
}

/// Read all Howick nodes — verifies address space structure is correct.
#[tokio::test]
async fn opcua_read_all_nodes() {
    let addr = start_opcua_server().await;
    let session = connect_client(addr, test_id()).await;
    let ns = get_namespace_index(&session, "urn:howick-frama")
        .await
        .unwrap_or(2);

    // Wait for first sync
    sleep(Duration::from_millis(600)).await;

    let node_names = [
        "Machine/Status",
        "Machine/CurrentJob",
        "Machine/PiecesProduced",
        "Machine/CoilRemaining",
        "Machine/LastError",
        "Jobs/QueueDepth",
        "Jobs/CompletedCount",
        "Jobs/PendingJobId",
        "Jobs/PendingJobName",
        "Jobs/PendingJobCsv",
    ];

    let read_ids: Vec<ReadValueId> = node_names
        .iter()
        .map(|n| ReadValueId {
            node_id: NodeId::new(ns, *n),
            attribute_id: AttributeId::Value as u32,
            ..Default::default()
        })
        .collect();

    let results = session
        .read(&read_ids, TimestampsToReturn::Both, 0.0)
        .await
        .unwrap();

    assert_eq!(results.len(), node_names.len());
    // Every node should have a value (not None)
    for (i, result) in results.iter().enumerate() {
        assert!(
            result.value.is_some(),
            "Node {} should have a value",
            node_names[i]
        );
    }

    let queue_depth = extract_u32(&results[5]).unwrap_or(999);
    let pending_id = extract_string(&results[7]).unwrap_or_default();

    assert_eq!(queue_depth, 0, "Queue should be empty at start");
    assert_eq!(pending_id, "", "No pending job at start");
}

/// Subscribe to Jobs/PendingJobId — verifies OPC UA subscription works.
/// This is the key M2M pattern: server pushes when a job arrives, no polling.
#[tokio::test]
async fn opcua_subscribe_pending_job_id() {
    let addr = start_opcua_server().await;
    let session = connect_client(addr, test_id()).await;
    let ns = get_namespace_index(&session, "urn:howick-frama")
        .await
        .unwrap_or(2);

    // Set up subscription — same as howick-agent does in production
    let received: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let received_clone = received.clone();

    let sub_id = session
        .create_subscription(
            Duration::from_millis(500),
            10,
            30,
            0,
            0,
            true,
            DataChangeCallback::new(move |dv: DataValue, _item: &MonitoredItem| {
                if let Some(Variant::String(s)) = dv.value {
                    let v = s.value().clone().unwrap_or_default();
                    received_clone.lock().unwrap().push(v);
                }
            }),
        )
        .await
        .unwrap();

    session
        .create_monitored_items(
            sub_id,
            TimestampsToReturn::Both,
            vec![MonitoredItemCreateRequest::from(NodeId::new(
                ns,
                "Jobs/PendingJobId",
            ))],
        )
        .await
        .unwrap();

    // Initial value push from subscription setup (usually "") — wait for it
    sleep(Duration::from_millis(800)).await;

    let notifications = received.lock().unwrap().clone();
    // Should have received at least the initial value (empty string on start)
    assert!(
        !notifications.is_empty(),
        "Should have received at least one notification (initial value)"
    );
}

// ── Helpers ────────────────────────────────────────────────────────────────────

async fn get_namespace_index(session: &opcua::client::Session, uri: &str) -> Option<u16> {
    use opcua::types::VariableId;

    let results = session
        .read(
            &[ReadValueId {
                node_id: VariableId::Server_NamespaceArray.into(),
                attribute_id: AttributeId::Value as u32,
                ..Default::default()
            }],
            TimestampsToReturn::Server,
            0.0,
        )
        .await
        .ok()?;

    if let Some(Variant::Array(arr)) = &results.first()?.value {
        arr.values.iter().enumerate().find_map(|(i, v)| {
            if let Variant::String(s) = v {
                if s.value().as_deref() == Some(uri) {
                    Some(i as u16)
                } else {
                    None
                }
            } else {
                None
            }
        })
    } else {
        None
    }
}

fn extract_string(dv: &DataValue) -> Option<String> {
    match &dv.value {
        Some(Variant::String(s)) => s.value().clone(),
        _ => None,
    }
}

fn extract_u32(dv: &DataValue) -> Option<u32> {
    match &dv.value {
        Some(Variant::UInt32(n)) => Some(*n),
        _ => None,
    }
}
