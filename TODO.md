# TODO

# Done

- Set up rust
- Integrate wasmi 
- Create a wasm from Assembly Script
- Run that wasm
- Linking host functions
- Count gas
- Gas limit

# Proof of Concept

- Simulate (as query call)
- Pass input / create a callback

# Smart Contract

- Request execution
- Receive callback
- ABI encode & decode

# Frontend

- Compile script
- Store wasm
- Simulate
- Run
- Store source (separately)

# Distributed System

- Run functions on an executor pool, rather than a single canister
- Allow calling other functions (including recursive calls)
- This will effectively allow for multi-threading
- Can also circumvent the ICP execution limit by yielding control (something like `await delay(0)`)
