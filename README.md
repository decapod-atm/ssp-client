# SSP Client

This binary crate is a basic reference client for communicating interactively with a SSP/eSSP server.

Currently, the only supported transport protocol is JSON-RPC.

## CLI mode

The client supports a very basic CLI mode, allowing the user to drop into a command shell.

To run the CLI:

```bash
cargo run [--release] -- -c
```

or

```
cargo build [--release]
./target/{debug,release}/ssp-client -c
```

## One-off mode

The client can also be used to send a single command to the server:

```bash
./target/{debug,release}/ssp-client -m status
```
