use std::{
    net::{IpAddr, SocketAddr},
    ops::Range,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use ambient_core::{asset_cache, FIXED_SERVER_TICK_TIME};
use ambient_ecs::{
    generated::network::components::no_sync, ArchetypeFilter, ComponentDesc, System, SystemGroup,
    World, WorldStream, WorldStreamCompEvent, WorldStreamFilter,
};
use ambient_native_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, ServerBaseUrlKey, UsingLocalDebugAssetsKey},
    fps_counter::FpsCounter,
    log_result, RUNTIME_USER_AGENT,
};
use ambient_proxy::client::AllocatedEndpoint;
use ambient_sys::time::Instant;
use anyhow::Context;
use colored::Colorize;
use futures::{SinkExt, StreamExt};
use parking_lot::{Mutex, RwLock};
use quinn::{ClientConfig, Connecting, Endpoint, ServerConfig, TransportConfig};
use rustls::{Certificate, PrivateKey};
use tokio::time::{interval, MissedTickBehavior};
use uuid::Uuid;

use crate::{
    native::{
        client_connection::ConnectionKind, load_root_certs, webtransport::handle_h3_connection,
    },
    proto::{
        server::{handle_diffs, ConnectionData, ServerProtoState},
        ServerInfo, ServerPush,
    },
    server::{
        server_stats, ForkingEvent, ProxySettings, ServerState, SharedServerState, ShutdownEvent,
        WorldInstance, MAIN_INSTANCE_ID,
    },
    stream::{FramedRecvStream, FramedSendStream},
    ServerWorldExt,
};

#[derive(Debug, Clone)]
pub struct Crypto {
    pub cert_chain: Vec<Vec<u8>>,
    pub key: Vec<u8>,
}

/// Quinn and Webtransport game server
pub struct GameServer {
    endpoint: Endpoint,
    /// Shuts down the server if there are no players
    pub inactivity_shutdown: Option<Duration>,
    proxy_settings: Option<ProxySettings>,
}

impl GameServer {
    pub async fn new_with_port(
        server_addr: SocketAddr,
        inactivity_shutdown: Option<Duration>,
        proxy_settings: Option<ProxySettings>,
        crypto: &Crypto,
    ) -> anyhow::Result<Self> {
        let endpoint = create_server(server_addr, crypto)?;

        tracing::debug!("GameServer listening on port {}", server_addr.port());
        Ok(Self {
            endpoint,
            inactivity_shutdown,
            proxy_settings,
        })
    }

    pub async fn new_with_port_in_range(
        bind_addr: IpAddr,
        port_range: Range<u16>,
        inactivity_shutdown: Option<Duration>,
        proxy_settings: Option<ProxySettings>,
        crypto: &Crypto,
    ) -> anyhow::Result<Self> {
        for port in port_range {
            match Self::new_with_port(
                SocketAddr::new(bind_addr, port),
                inactivity_shutdown,
                proxy_settings.clone(),
                crypto,
            )
            .await
            {
                Ok(server) => {
                    return Ok(server);
                }
                Err(err) => {
                    tracing::warn!("Failed to create server on port {port}.\n{err:?}");
                }
            }
        }
        anyhow::bail!("Failed to create server")
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn run(
        self,
        mut world: World,
        create_server_systems: Arc<dyn Fn(&mut World) -> SystemGroup + Sync + Send>,
        create_on_forking_systems: Arc<dyn Fn() -> SystemGroup<ForkingEvent> + Sync + Send>,
        create_shutdown_systems: Arc<dyn Fn() -> SystemGroup<ShutdownEvent> + Sync + Send>,
        is_sync_component: Arc<dyn Fn(ComponentDesc, WorldStreamCompEvent) -> bool + Sync + Send>,
        on_server_state_created: Arc<dyn Fn(SharedServerState) + Sync + Send>,
    ) -> SharedServerState {
        let Self {
            endpoint,
            proxy_settings,
            ..
        } = self;

        let assets = world.resource(asset_cache()).clone();
        let world_stream_filter =
            WorldStreamFilter::new(ArchetypeFilter::new().excl(no_sync()), is_sync_component);
        let state = Arc::new(Mutex::new(ServerState::new(
            assets.clone(),
            [(
                MAIN_INSTANCE_ID.to_string(),
                WorldInstance {
                    systems: create_server_systems(&mut world),
                    world,
                    world_stream: WorldStream::new(world_stream_filter.clone()),
                },
            )]
            .into_iter()
            .collect(),
            create_server_systems,
            create_on_forking_systems,
            create_shutdown_systems,
        )));
        on_server_state_created(state.clone());

        let mut fps_counter = FpsCounter::new();
        let mut sim_interval = interval(FIXED_SERVER_TICK_TIME);
        sim_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        let mut inactivity_interval = interval(Duration::from_secs_f32(5.));
        let mut last_active = ambient_sys::time::Instant::now();

        if let Some(proxy_settings) = proxy_settings {
            let endpoint = endpoint.clone();
            let state = state.clone();
            let world_stream_filter = world_stream_filter.clone();
            let assets = assets.clone();
            tokio::spawn(async move {
                start_proxy_connection(
                    endpoint.clone(),
                    proxy_settings,
                    state.clone(),
                    world_stream_filter.clone(),
                    assets.clone(),
                )
                .await;
            });
        }

        loop {
            let addr = endpoint.local_addr().unwrap();

            tracing::trace_span!("Listening for incoming connections", ?addr,);
            tokio::select! {
                Some(conn) = endpoint.accept() => {
                    let fut = resolve_connection(conn, state.clone(), world_stream_filter.clone(), ServerBaseUrlKey.get(&assets));
                    tokio::spawn(async move {  log_result!(fut.await) });
                }
                _ = sim_interval.tick() => {
                    fps_counter.frame_start();
                    let mut state = state.lock();
                    tokio::task::block_in_place(|| {
                        profiling::finish_frame!();
                        profiling::scope!("sim_tick");
                        state.step();
                        state.broadcast_diffs();
                        if let Some(sample) = fps_counter.frame_end() {
                            for instance in state.instances.values_mut() {
                                let id = instance.world.synced_resource_entity().unwrap();
                                instance.world.add_component(id, server_stats(), sample.clone()).unwrap();
                            }
                        }
                    });
                }
                _ = inactivity_interval.tick(), if self.inactivity_shutdown.is_some() => {
                    if state.lock().player_count() == 0 {
                        if Instant::now().duration_since(last_active) > self.inactivity_shutdown.unwrap() {
                            tracing::info!("Shutting down due to inactivity");
                            break;
                        }
                    } else {
                        last_active = Instant::now();
                    }
                }
                else => {
                    tracing::info!("No more connections. Shutting down.");
                    break
                }
            }
        }
        tracing::debug!("GameServer shutting down");
        {
            let mut state = state.lock();
            let create_shutdown_systems = state.create_shutdown_systems.clone();
            for instance in state.instances.values_mut() {
                let mut sys = (create_shutdown_systems)();
                sys.run(&mut instance.world, &ShutdownEvent);
            }
        }
        tracing::debug!("GameServer finished shutting down");
        state
    }

    /// Returns the local socket address of the endpoint
    pub fn local_addr(&self) -> SocketAddr {
        self.endpoint
            .local_addr()
            .expect("Failed go get socket address for endpoint")
    }
}

async fn resolve_connection(
    mut conn: Connecting,
    state: SharedServerState,
    world_stream_filter: WorldStreamFilter,
    content_base_url: AbsAssetUrl,
) -> anyhow::Result<()> {
    tracing::debug!("Received connection");

    let handshake_data = conn
        .handshake_data()
        .await
        .context("Failed to acquire handshake data")?
        .downcast::<quinn::crypto::rustls::HandshakeData>()
        .ok()
        .context("Failed to downcast handshake data")?;

    let protocol = handshake_data.protocol.context("Missing protocol")?;
    let protocol_str = std::str::from_utf8(&protocol);
    let server_name = handshake_data.server_name;

    tracing::trace!(
        ?protocol,
        ?protocol_str,
        ?server_name,
        "Received handshake data"
    );

    let conn = match conn.await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to accept incoming connection. {e}");
            return Ok(());
        }
    };

    tracing::debug!("Accepted connection");
    if protocol == b"ambient-02" {
        handle_quinn_connection(
            conn.into(),
            state.clone(),
            world_stream_filter.clone(),
            content_base_url,
        )
        .await
    } else if protocol == b"h3" {
        handle_h3_connection(conn, state.clone(), world_stream_filter, content_base_url).await
    } else {
        tracing::error!(
            local_ip=?conn.local_ip(),
            "Client connected using unsupported protocol: {:?}",
            protocol
        );

        conn.close(0u32.into(), b"Unsupported protocol");
        Ok(())
    }
}

/// Setup the protocol and enter the update loop for a new connected client
#[tracing::instrument(level = "info", skip_all, fields(content_base_url))]
async fn handle_quinn_connection(
    conn: ConnectionKind,
    state: SharedServerState,
    world_stream_filter: WorldStreamFilter,
    content_base_url: AbsAssetUrl,
) -> anyhow::Result<()> {
    tracing::debug!("Handling server connection");

    if !conn.is_local() && UsingLocalDebugAssetsKey.get(&state.lock().assets) {
        tracing::warn!("Client connected from remote address but server is using debug assets. This might involve uploading large files to the client.");
    }

    let (diffs_tx, diffs_rx) = flume::unbounded();

    let server_info = ServerInfo::new(&mut state.lock(), content_base_url);

    let mut server = ServerProtoState::default();

    let mut request_recv = FramedRecvStream::new(conn.accept_uni().await?);
    let mut push_send = FramedSendStream::new(conn.open_uni().await?);

    // Send who we are
    push_send.send(ServerPush::ServerInfo(server_info)).await?;

    // Feed the channel senders to the connection data
    //
    // Once connected they will be added to the player entity
    let data = ConnectionData {
        conn: Arc::new(conn.clone()),
        state,
        diff_tx: diffs_tx,
        connection_id: Uuid::new_v4(),
        world_stream_filter,
    };

    while server.is_pending_connection() {
        if let Some(frame) = request_recv.next().await {
            server.process_control(&data, frame?)?;
        }
    }

    tokio::spawn(handle_diffs(
        FramedSendStream::new(conn.open_uni().await?),
        diffs_rx,
    ));

    let mut server = scopeguard::guard(server, |mut server| {
        if !server.is_disconnected() {
            tracing::info!("Connection closed abruptly from {server:?}");
            server.process_disconnect(&data);
        }
    });

    // Before a connection has been established, only process the control stream
    while let ServerProtoState::Connected(connected) = &mut *server {
        tokio::select! {
            Some(frame) = request_recv.next() => {
                server.process_control(&data, frame?)?;
            }
            stream = conn.accept_uni() => {
                connected.process_uni(&data, stream?);
            }
            stream = conn.accept_bi() => {
                let (send, recv) = stream?;
                connected.process_bi(&data, send, recv);
            }
            datagram = conn.read_datagram() => {
                connected.process_datagram(&data, datagram?)?;
            }
            Some(msg) = connected.control_rx.next() => {
                push_send.send(&msg).await?;
            }
        }
    }

    tracing::info!("Client disconnected");

    Ok(())
}

async fn start_proxy_connection(
    endpoint: Endpoint,
    settings: ProxySettings,
    state: SharedServerState,
    world_stream_filter: WorldStreamFilter,
    assets: AssetCache,
) {
    // start with content base url being the same as for direct connections
    let content_base_url = Arc::new(RwLock::new(ServerBaseUrlKey.get(&assets)));

    let on_endpoint_allocated = {
        let content_base_url = content_base_url.clone();
        Arc::new(
            move |AllocatedEndpoint {
                      id,
                      allocated_endpoint,
                      external_endpoint,
                      assets_root,
                      ..
                  }: AllocatedEndpoint| {
                tracing::debug!("Allocated proxy endpoint. Allocation id: {}", id);
                tracing::debug!("Proxy sees this server as {}", external_endpoint);
                tracing::info!(
                    "Proxy allocated an endpoint, use `{}` to join",
                    format!("ambient join {}", allocated_endpoint).bright_green()
                );

                // set the content base url to point to proxy provided value
                match AbsAssetUrl::from_str(&assets_root) {
                    Ok(url) => {
                        tracing::debug!("Got content base root from proxy: {}", url);
                        *content_base_url.write() = url;
                    }
                    Err(err) => {
                        tracing::warn!("Failed to parse assets root url ({}): {}", assets_root, err)
                    }
                }
            },
        )
    };

    let on_player_connected = {
        let content_base_url = content_base_url.clone();
        Arc::new(
            move |_player_id, conn: ambient_proxy::client::ProxiedConnection| {
                tracing::debug!("Accepted connection via proxy");
                let task = handle_quinn_connection(
                    conn.into(),
                    state.clone(),
                    world_stream_filter.clone(),
                    content_base_url.read().clone(),
                );

                tokio::spawn(async move { log_result!(task.await) });
            },
        )
    };

    let builder = ambient_proxy::client::builder()
        .endpoint(endpoint.clone())
        .proxy_server(settings.endpoint.clone())
        .project_id(settings.primary_package_id.clone())
        .user_agent(RUNTIME_USER_AGENT.to_string());

    let assets_path = settings.build_path;
    let builder = if let Ok(Some(assets_file_path)) = assets_path.to_file_path() {
        builder.assets_path(assets_file_path)
    } else {
        builder.assets_root_override(content_base_url.read().to_string())
    };

    tracing::info!("Connecting to proxy server");
    let proxy = match builder.build().await {
        Ok(proxy_client) => proxy_client,
        Err(err) => {
            tracing::warn!("Failed to connect to proxy: {}", err);
            return;
        }
    };

    // start and allocate endpoint
    let mut controller = proxy.start(on_endpoint_allocated, on_player_connected);
    if let Err(err) = controller.allocate_endpoint().await {
        tracing::warn!("Failed to allocate proxy endpoint: {}", err);
    }

    // pre-cache "assets" subdirectory
    if settings.pre_cache_assets {
        for subdir in ["assets", "client"] {
            if let Err(err) = controller.pre_cache_assets(subdir) {
                tracing::warn!("Failed to pre-cache assets: {}", err);
            }
        }
    }
}

fn create_server(server_addr: SocketAddr, crypto: &Crypto) -> anyhow::Result<Endpoint> {
    let mut tls_config = rustls::ServerConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&rustls::version::TLS13])
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(
            crypto.cert_chain.iter().cloned().map(Certificate).collect(),
            PrivateKey(crypto.key.clone()),
        )?;

    tls_config.max_early_data_size = u32::MAX;
    let alpn: Vec<Vec<u8>> = vec![
        b"h3".to_vec(),
        b"h3-32".to_vec(),
        b"h3-31".to_vec(),
        b"h3-30".to_vec(),
        b"h3-29".to_vec(),
        b"ambient-02".to_vec(),
    ];

    tls_config.alpn_protocols = alpn;

    let mut server_conf = ServerConfig::with_crypto(Arc::new(tls_config));
    let mut transport = TransportConfig::default();

    transport.keep_alive_interval(Some(Duration::from_secs(2)));

    if std::env::var("AMBIENT_DISABLE_TIMEOUT").is_ok() {
        transport.max_idle_timeout(None);
    } else {
        transport.max_idle_timeout(Some(Duration::from_secs_f32(60.).try_into()?));
    }

    let transport = Arc::new(transport);
    server_conf.transport = transport.clone();

    let mut endpoint = Endpoint::server(server_conf, server_addr)?;

    // Create client config for the server endpoint for proxying and hole punching
    let mut roots = load_root_certs();

    // add proxy test cert if provided
    if let Ok(test_ca_cert_path) = std::env::var("AMBIENT_PROXY_TEST_CA_CERT") {
        let cert = Certificate(std::fs::read(test_ca_cert_path)?);
        roots.add(&cert).unwrap();
    }

    let mut crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();
    crypto.alpn_protocols = vec![b"ambient-proxy-03".to_vec()];

    let mut client_config = ClientConfig::new(Arc::new(crypto));
    client_config.transport_config(transport);
    endpoint.set_default_client_config(client_config);

    Ok(endpoint)
}
