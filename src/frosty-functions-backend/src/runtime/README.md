# Runtime Module

This module contains the core logic for running a Frosty WASM module. All code in
this module is designed to work in synchronous contexts as well (e.g. in a simulation).
Callers need to implement the `FrostyEnv` trait with any asynchronous calls and
scheduling logic.
