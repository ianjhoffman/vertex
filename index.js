import init, { run } from './pkg/vertex.js';

async function run_wasm() {
    await init();
    fetch('/puzzles/1.txt')
        .then((res) => res.text())
        .then((text) => {
            run(text);
        });
}

run_wasm();
