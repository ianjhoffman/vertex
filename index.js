import init, { run } from './pkg/vertex.js';

async function run_wasm() {
    await init();
    run();
}

run_wasm();
