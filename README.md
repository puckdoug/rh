# rh: an experimental AI harness written in Rust

This is currently an experiment to create a harness for running AI agents in rust (rh: Rust Harness). Goals beyond what standard harnesses provide:

1. Allow async communication (e.g. the harness should accept input even when working on a task)
2. Allow extensive introspection so the harness can answer questions regarding its own operation.
3. It's rust, so of course run fast and stable.

The harness will manage a suite of specialized agents to deliver complete responses. This will need async (Tokio) interaction on top of the interaction with the user.
