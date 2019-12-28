import init, { run } from './pkg/vertex.js';

async function run_wasm() {
    await init();
    var puzzle = window.prompt("Select a puzzle number", "1");
    fetch(`/puzzles/${puzzle}.txt`)
        .then((res) => res.text())
        .then((text) => {
            run(text);
        });
}

run_wasm();
