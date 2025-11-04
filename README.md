# webgpu-tutorial

This is a repo containing [wgpu](https://crates.io/crates/wgpu) examples.

Most examples are tested on the latest chrome browser.

## basic

WebGPU running on WASM and bundling with Webpack.

> [Go to the source code](basic/src/lib.rs)

## layout

A test file to see when memory layout errors occur and how to resolve them.

> [Go to the source code](layout/src/lib.rs)

## offscreen-on-worker

Drawing on web worker using offscreen canvas and wgpu.

> [Go to the source code](offscreen-on-worker/src/lib.rs)

## composite-shader

Test using naga_oil to compose shader sources.

> [Go to the source code](composite-shader/src/lib.rs)

## webgpu-or-webgl

Check browser support out and choose WebGPU or WebGL.

> [Go to the source code](webgpu-or-webgl/src/lib.rs)

## wgpu-gl-surfaces

Trouble shooting for multiple surfaces of wgpu WebGL2 context.

> [Go to the source code](wgpu-gl-surfaces/src/lib.rs)
