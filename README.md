# Websocket RPC

## Purpose

I need an RPC-endpoint that looks like a websocket client communicating to some public address.

This crate defines the `Agent` and the `Router`. `Agent`s send and receive messages to each other
via the `Router`. So some agents can be made RPC endpoints and others can be made RPC clients.
The `Router` will be a public websocket server.