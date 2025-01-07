use crate::{
    client::{CleanupFunc, ClientState, Control, GameClientRenderTarget, LoadedFunc, NetworkStats},
    client_game_state::{game_screen_render_target, ClientGameState},
    native::load_root_certs,
    proto::{
        client::{ClientProtoState, SharedClientGameState},
        ClientRequest,
    },
    server::RpcArgs,
    stream::{FramedRecvStream, FramedSendStream, RawFramedRecvStream},
    NetworkError,
};
use ambient_app::{window_title, world_instance_resources, AppResources};
use ambient_core::{asset_cache, gpu};
use ambient_ecs::{generated::messages, world_events, Entity, SystemGroup};
use ambient_element::{
    consume_context, provide_context, use_frame, use_runtime_message, use_state, use_state_with,
    use_task, Element, ElementComponent, ElementComponentExt, Hooks,
};
use ambient_native_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    asset_url::ContentBaseUrlKey,
    Cb,
};
use ambient_renderer::RenderTarget;
use ambient_rpc::RpcRegistry;
use ambient_ui_native::{Centered, Dock, FlowColumn, FlowRow, StylesExt, Text, Throbber};
use anyhow::Context;
use futures::{SinkExt, StreamExt};
use glam::uvec2;
use parking_lot::Mutex;
use quinn::{ClientConfig, Connection, Endpoint, TransportConfig};
use rand::Rng;
use rustls::Certificate;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};
use tokio::net::ToSocketAddrs;

#[derive(Debug, Clone)]
pub struct ResolvedAddr {
    pub host_name: String,
    pub addr: SocketAddr,
}

impl ResolvedAddr {
    pub async fn lookup_host<T: ToSocketAddrs + ToString + Clone>(host: T) -> anyhow::Result<Self> {
        let addr = tokio::net::lookup_host(host.clone())
            .await?
            .find(SocketAddr::is_ipv4)
            .ok_or_else(|| anyhow::anyhow!("No IPv4 addresses found for: {}", host.to_string()))?;
        let host = host.to_string();
        let host_name = if host.contains(':') {
            host.split(':').next().unwrap().to_string()
        } else {
            host
        };
        Ok(Self { host_name, addr })
    }

    pub fn localhost_with_port(port: u16) -> Self {
        Self {
            host_name: "localhost".into(),
            addr: ([127, 0, 0, 1], port).into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientView {
    pub server_addr: ResolvedAddr,
    pub cert: Option<Vec<u8>>,
    pub user_id: String,
    pub fail_on_version_mismatch: bool,
    pub systems_and_resources: Cb<dyn Fn() -> (SystemGroup, Entity) + Sync + Send>,
    pub on_loaded: LoadedFunc,
    pub create_rpc_registry: Cb<dyn Fn() -> RpcRegistry<RpcArgs> + Sync + Send>,
    pub inner: Element,
}

impl ElementComponent for ClientView {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self {
            server_addr,
            user_id,
            fail_on_version_mismatch,
            systems_and_resources,
            create_rpc_registry,
            on_loaded,
            inner,
            cert,
        } = *self;

        let gpu = hooks.world.resource(gpu()).clone();

        provide_context(hooks, || {
            GameClientRenderTarget(Arc::new(RenderTarget::new(&gpu, uvec2(1, 1), None)))
        });

        let (render_target, _) = consume_context::<GameClientRenderTarget>(hooks).unwrap();

        let assets = hooks.world.resource(asset_cache()).clone();

        let ((control_tx, control_rx), _) = use_state_with(hooks, |_| flume::unbounded());

        // The game client will be set once a connection establishes
        let (client_state, set_client_state) = use_state(hooks, None as Option<ClientState>);

        // Subscribe to window close events
        use_runtime_message::<messages::WindowClose>(hooks, {
            move |_, _| {
                tracing::debug!("User closed the window");
                control_tx.send(Control::Disconnect).ok();
            }
        });

        // Run game logic
        {
            let gpu = gpu.clone();
            let render_target = render_target.clone();
            let world_event_reader = Mutex::new(hooks.world.resource(world_events()).reader());

            let client_state = client_state.clone();
            use_frame(hooks, move |app_world| {
                if let Some(client_state) = &client_state {
                    let mut game_state = client_state.game_state.lock();
                    // Pipe events from app world to game world
                    for (_, event) in world_event_reader
                        .lock()
                        .iter(app_world.resource(world_events()))
                    {
                        game_state
                            .world
                            .resource_mut(world_events())
                            .add_event(event.clone());
                    }

                    game_state.on_frame(&gpu, &render_target.0);
                } else {
                    tracing::debug!("No game state");
                }
            });
        }

        // Set the window title to the package name
        let (window_title_state, set_window_title) = use_state(hooks, "Ambient".to_string());
        *hooks.world.resource_mut(window_title()) = window_title_state;

        let (err, set_error) = use_state(hooks, None);

        use_task(hooks, move |ui_world| {
            let local_resources = world_instance_resources(AppResources::from_world(ui_world))
                .with(game_screen_render_target(), render_target.0.clone());
            let task = async move {
                let conn = open_connection(server_addr.clone(), cert.map(Certificate))
                    .await
                    .with_context(|| {
                        format!("Failed to connect to endpoint \"{server_addr:?}\"")
                    })?;

                handle_connection(
                    conn.clone(),
                    &assets,
                    user_id,
                    fail_on_version_mismatch,
                    move |args| {
                        let OnConnectionState {
                            assets,
                            user_id,
                            main_package_name,
                        } = args;

                        set_window_title(main_package_name.to_string());

                        let (systems, resources) = systems_and_resources();
                        let resources = local_resources
                            .clone()
                            .with(ambient_core::player::local_user_id(), user_id.into())
                            .with_merge(resources);

                        let game_state = ClientGameState::new(
                            &gpu,
                            assets.clone(),
                            user_id.into(),
                            systems,
                            resources,
                        );

                        // Create a handle for the game client
                        let client_state = ClientState::new(
                            Arc::new(conn.clone()),
                            Arc::new(create_rpc_registry()),
                            Arc::new(Mutex::new(game_state)),
                            user_id.into(),
                        );

                        let game_state = &client_state.game_state;
                        let cleanup = {
                            // Lock before setting
                            let game_state = &mut game_state.lock();

                            // Updates the game client context in the Ui tree
                            // Update the resources on the client side world to reflect the new connection
                            // state

                            game_state.world.add_resource(
                                crate::client::client_state(),
                                Some(client_state.clone()),
                            );

                            (on_loaded)(&client_state, game_state)?
                        };

                        // Set the client last so that the game state is initialized first
                        set_client_state(Some(client_state.clone()));

                        Ok((game_state.clone(), cleanup))
                    },
                    // game_state,
                    control_rx,
                )
                .await?;

                Ok(()) as anyhow::Result<()>
            };

            async move {
                match task.await {
                    Ok(()) => {
                        tracing::info!("Client disconnected");
                    }
                    Err(err) => {
                        if let Some(err) = err.downcast_ref::<NetworkError>() {
                            if let NetworkError::ConnectionClosed = err {
                                tracing::info!("Connection closed by peer");
                            } else {
                                tracing::error!("Network error: {:?}", err);
                            }
                        } else {
                            tracing::error!("Game failed: {:?}", err);
                        }
                        set_error(Some(format!("{err:?}")));
                    }
                }
            }
        });

        if let Some(err) = err {
            return Dock(vec![Text::el("Error").header_style(), Text::el(err)]).el();
        }

        if let Some(client_state) = &client_state {
            // Provide the context
            provide_context(hooks, || client_state.clone());
            hooks
                .world
                .add_resource(crate::client::client_state(), Some(client_state.clone()));

            inner
        } else {
            Centered(vec![FlowColumn::el([FlowRow::el([
                Text::el("Connecting"),
                Throbber.el(),
            ])])])
            .el()
        }
    }
}

struct OnConnectionState<'a> {
    assets: &'a AssetCache,
    user_id: &'a str,
    main_package_name: &'a str,
}

async fn handle_connection(
    conn: quinn::Connection,
    assets: &AssetCache,
    user_id: String,
    fail_on_version_mismatch: bool,
    mut on_loaded: impl FnMut(OnConnectionState) -> anyhow::Result<(SharedClientGameState, CleanupFunc)>
        + Send
        + Sync,
    control_rx: flume::Receiver<Control>,
) -> anyhow::Result<()> {
    let mut request_send = FramedSendStream::new(conn.open_uni().await?);

    // Accept the diff and stat stream
    // Nothing is read from them until the connection has been accepted

    // Send a connection request
    tracing::info!("Attempting to connect using {user_id:?}");

    request_send
        .send(ClientRequest::Connect(user_id.clone()))
        .await?;

    let mut client = ClientProtoState::Pending(user_id.clone());

    let mut push_recv = FramedRecvStream::new(conn.accept_uni().await?);

    while client.is_pending() {
        if let Some(frame) = push_recv.next().await {
            client.process_push(assets, fail_on_version_mismatch, frame?)?;
        }
    }

    assert!(ContentBaseUrlKey.exists(assets));

    let main_package_name = if let ClientProtoState::Connected(connected) = &client {
        connected.main_package_name.clone()
    } else {
        tracing::warn!("Connection failed or was denied");
        return Ok(());
    };

    tracing::info!("Connection successfully established");

    // Create the game client

    let mut diff_stream = RawFramedRecvStream::new(conn.accept_uni().await?);

    let (shared_client_state, cleanup) = on_loaded(OnConnectionState {
        assets,
        user_id: &user_id,
        main_package_name: &main_package_name,
    })?;

    let on_disconnect = move || {
        tracing::debug!("Running connection cleanup");
        cleanup()
    };

    scopeguard::defer!(on_disconnect());

    let stats_interval = 5;
    let mut stats_timer = tokio::time::interval(Duration::from_secs_f32(stats_interval as f32));
    let mut prev_stats = conn.stats();

    let mut control_rx = control_rx.into_stream();

    while let ClientProtoState::Connected(connected) = &mut client {
        tokio::select! {
            Some(frame) = push_recv.next() => {
                client.process_push(assets, fail_on_version_mismatch, frame?)?;
            }
            _ = stats_timer.tick() => {
                let stats = conn.stats();

                client.process_client_stats(&shared_client_state, NetworkStats {
                    latency_ms: conn.rtt().as_millis() as u64,
                    bytes_sent: (stats.udp_tx.bytes - prev_stats.udp_tx.bytes) / stats_interval,
                    bytes_received: (stats.udp_rx.bytes - prev_stats.udp_rx.bytes) / stats_interval,
                    packets_sent: (stats.path.sent_packets - prev_stats.path.sent_packets),
                    packets_lost: (stats.path.lost_packets - prev_stats.path.lost_packets),
                });

                prev_stats = stats;
            }

           Some(control) = control_rx.next() => {
                match control {
                    Control::Disconnect => {
                        tracing::debug!("Disconnecting manually");
                        // Tell the server that we want to gracefully disconnect
                        request_send.send(ClientRequest::Disconnect).await?;
                    }
                }
            }

            Ok(datagram) = conn.read_datagram() => {
                connected.process_datagram(&shared_client_state, datagram)?;
            }
            Ok((send, recv)) = conn.accept_bi() => {
                connected.process_bi(&shared_client_state, send, recv);
            }
            Ok(recv) = conn.accept_uni() => {
                connected.process_uni(&shared_client_state, recv);
            }
            Some(diff) = diff_stream.next() => {
                connected.process_diff(&shared_client_state, diff?)?;
            }
        }
    }

    Ok(())
}

/// Connnect to the server endpoint.
#[tracing::instrument(level = "debug", skip(cert))]
async fn open_connection(
    server_addr: ResolvedAddr,
    cert: Option<Certificate>,
) -> anyhow::Result<Connection> {
    tracing::debug!("Connecting to world instance: {server_addr:?}");

    let endpoint =
        create_client_endpoint_random_port(cert).context("Failed to create client endpoint")?;

    tracing::debug!("Got endpoint");
    let conn = endpoint
        .connect(server_addr.addr, &server_addr.host_name)?
        .await?;

    tracing::debug!("Got connection");
    Ok(conn)
}

pub fn create_client_endpoint_random_port(cert: Option<Certificate>) -> anyhow::Result<Endpoint> {
    let mut roots = load_root_certs();

    if let Some(cert) = cert {
        roots
            .add(&cert)
            .context("Failed to add custom certificate")?;
    }

    for _ in 0..10 {
        let client_port = {
            let mut rng = rand::thread_rng();
            rng.gen_range(15000..25000)
        };

        let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), client_port);

        if let Ok(mut endpoint) = Endpoint::client(client_addr) {
            let mut tls_config = rustls::ClientConfig::builder()
                .with_safe_default_cipher_suites()
                .with_safe_default_kx_groups()
                .with_protocol_versions(&[&rustls::version::TLS13])
                .unwrap()
                .with_root_certificates(roots)
                .with_no_client_auth();

            // tls_config.enable_early_data = true;
            tls_config.alpn_protocols = vec!["ambient-02".into()];

            let mut transport = TransportConfig::default();
            transport.keep_alive_interval(Some(Duration::from_secs_f32(1.)));

            if std::env::var("AMBIENT_DISABLE_TIMEOUT").is_ok() {
                transport.max_idle_timeout(None);
            } else {
                transport.max_idle_timeout(Some(Duration::from_secs_f32(60.).try_into().unwrap()));
            }
            let mut client_config = ClientConfig::new(Arc::new(tls_config));
            client_config.transport_config(Arc::new(transport));

            endpoint.set_default_client_config(client_config);
            return Ok(endpoint);
        }
    }

    Err(anyhow::anyhow!(
        "Failed to find appropriate port for client endpoint"
    ))
}
