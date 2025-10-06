use wasmi::*;

fn run_wasm() -> Result<i64, String> {
    let wasm = include_bytes!("../../../../ic-wasmi-benchmark/target/wasm32-unknown-unknown/wasm/bench.wasm");
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm[..]).unwrap();
    let linker = <wasmi::Linker<()>>::new(module.engine());
    let mut store = wasmi::Store::new(module.engine(), ());
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .start(&mut store)
        .unwrap();

    let run = instance.get_typed_func::<(), i64>(&store, "run").unwrap();
    Ok(run.call(&mut store, ()).unwrap())
}

#[ic_cdk::query]
fn runWasm() -> String {
    let sum = run_wasm().unwrap();
    format!(
        "instructions: {}, sum: {}",
        ic_cdk::api::performance_counter(0),
        sum
    )
}

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
