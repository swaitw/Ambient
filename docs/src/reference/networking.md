# Networking

Networking is a critical component of Ambient, as it enables communication between the client and the server. This document explains some of the specifics behind the current protocol.

## Protocol

Currently, the Ambient runtime supports both desktop and web clients, using QUIC through the `quinn` library/WebTransport through `h3-webtransport` respectively as its communication protocol.

The HTTP (TCP) port is `8999`, and the QUIC (UDP) port is `9000`.

## Entities

The Ambient runtime synchronizes all entities by default. Only components marked as `Networked` will be sent to the client. Most core components are `Networked`, but custom components are not by default; this is something developers have to opt into. It is important to note that this may have unintended ramifications in terms of cheating, especially for hostile clients.

To disable syncing an entity to the client, attach the `no_sync` component to it. This will prevent the entity from being sent to the client.

The client is fundamentally designed around runtime flexibility of logic, which is non-ideal for avoiding cheaters. Further research and development are required, but it is likely that there is no silver bullet, and the solution will be game-dependent.

### Entity synchronization

The Ambient runtime synchronizes entities using a diff-based approach. The server sends a `WorldDiff` to the client, which contains a list of entities to spawn and despawn, and components to add, update, and remove.

Note that some operations might be batched for performance or not included in the update sent to the clients if there is no effective change in value. For example, adding 0 to a number or changing a boolean to `false` and back to `true` within the same frame might not emit an update and might not trigger a `change_query`. We recommend using messaging if such events are important to your game.

Currently, the client applies the changes to its local world as soon as they are received.

## Logic and Prediction

All gameplay logic is currently server-authoritative. We currently do not have any form of latency-hiding, including prediction, rollback, or clientside logic. We have previously experimented with rollback, but it was removed due to difficulties in genericising its implementation, as the solution would have to be different for each class of game.

Our plan is to continue improving our data model to enable user-defined prediction, provided as an Ambient package, but this work is ongoing. In the meantime, prediction can be done manually by sharing code with some caveats (i.e. physics does not run on the client).

## Messaging

The Ambient runtime supports messaging from the client to the server and vice versa through structured messages. These messages are defined ahead of time in `ambient.toml` and made accessible to code that consumes that `ambient.toml`.

This messaging can be reliable (QUIC unistream) or unreliable (QUIC datagram). Developers can use this to define their networked behavior, including customized prediction.

See [the messages reference](./messages.md) for more details.

## Proxy

From 0.2 onwards, Ambient will establish a connection to a NAT traversal proxy by default (this can be turned off with `--no-proxy`). This proxy allows users to connect to an Ambient server, even when the server is behind NAT or similar. Check the [AmbientProxy repository](https://github.com/AmbientRun/AmbientProxy) for more details about the proxy itself.

The Ambient server (i.e. Ambient when started with `run` or `serve`) connects to the proxy using QUIC (using the `quinn` library) and allocates a proxy endpoint. In response, the proxy provides the endpoint's details as well as an URL for asset downloading. The allocated proxy endpoint can be used by players to connect (`ambient join ...`) to the game server, even if it is running behind a NAT.

Communication between the proxy and players uses the same protocol as with a direct connection to the Ambient server; the only difference is the proxy acting as an intermediary.

## Certificates

By default, Ambient bundles a self-signed certificate that is used by the server and trusted by the client. This enables connecting to the server without any additional configuration, but may limit direct connections from other clients.

We recommend use of the proxy, or using your own certificate. In future, we may offer an option to relax certificate verification for native servers and clients.

To use your own certificate:

- specify `--cert` and `--key` for the server:
  ```sh
  ambient serve --cert ./localhost.crt --key ./localhost.key
  ```
- specify `--ca` for the client if the certificate authority that signed the certificate is not present within the client's system roots
  ```sh
  ambient join 127.0.0.1:9000
  ```

If a custom certificate is specified, the bundled certificates will _not_ be used as a fallback.
