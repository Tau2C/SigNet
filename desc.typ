#set page(paper: "a4")

= P2P VPN

== Overview
A custom-built tunnel system allowing remote access to a home machine from a mobile device (e.g., tablet) even behind NATs.
Designed to work on *NixOS* (home laptop) and *Termux/Nix-on-Droid* (tablet).

== Objectives

- Provide remote access to SSH and arbitrary TCP services (e.g., web server on port 80).
- Enable direct peer-to-peer (P2P) connections when possible.
- Fall back to encrypted relay when NAT traversal fails.
- Ensure full end-to-end encryption and mutual authentication.
- Maintain lightweight footprint and cross-platform portability.

== Architecture

=== Components

- *Agent:*
  Runs on each device (home + tablet). Maintains persistent key pair, handles connections, forwards ports, or exposes a SOCKS5 proxy.
- *Broker:*
  A lightweight public server used for signaling and optional relaying.
  Handles authentication, message relay, and optional TCP proxying.

=== Connection Flow

1. Both clients connect to the broker via TLS or WebSocket.
2. Each client registers with a public key and capabilities.
3. When one client requests connection to another, the broker performs signaling.
4. The clients attempt to establish a direct P2P channel using WebRTC or QUIC.
5. If P2P fails, traffic is relayed through the broker.

== Technologies

- *Language:* Go or Rust
- *Transport:* WebRTC (via `pion`) or libp2p (QUIC/TCP)
- *Encryption:* Ed25519 + Noise / TLS
- *Multiplexing:* `yamux` or `smux` for virtual streams

== Security Model

- Each client holds a persistent Ed25519 keypair.
- Authentication by challenge-response signature.
- All data encrypted end-to-end.
- Access control lists (ACLs) define allowed peers and forwarded ports.

== Features

- Persistent outbound-only connections (NAT-safe).
- Multi-port forwarding (SSH, HTTP, etc.).
- SOCKS5 proxy support for general browsing.
- Auto-reconnect and keepalive.
- Configurable via JSON or TOML.

== Implementation Plan

=== Phase 1 — Core Tunnel

- Implement broker with WebSocket signaling.
- Implement agents capable of TCP relaying through broker.
- Verify basic connectivity (tablet → home SSH).

=== Phase 2 — P2P Support

- Add WebRTC ICE/STUN to attempt direct connections.
- Integrate fallback relay for non-traversable NATs.

=== Phase 3 — Security and Config

- Implement Ed25519 identity system and signed authentication.
- Add ACL configuration and encrypted config files.

=== Phase 4 — Multiplexing

- Integrate `yamux` for multi-stream tunneling.
- Add SOCKS5 proxy support.

=== Phase 5 — Packaging

- NixOS systemd service for home agent.
- Termux autostart support for mobile agent.

== Example Use Case

- Home laptop exposes:
  - SSH (port 22)
  - Local web dashboard (port 8080)
- Tablet connects through broker → automatically negotiates P2P.
- User browses to `http://localhost:8080` or runs `ssh home`.

== Future Enhancements

- Distributed broker network or DHT-based discovery.
- File synchronization over tunnel.
- Integration with existing VPN stacks (WireGuard backend).
