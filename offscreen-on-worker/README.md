# Drawing on web worker using wgpu & offscreen canvas

This example shows how to use offscreen canvas and web worker to draw something using wgpu.
In this example, shared memory isn't used because it requires unstable rust build and additional server side settings at the time of writing.  
To put heavy jobs on the worker, data relative to drawing are on worker side.
And it receives window message from main thread.

This example builds wasm with '--target web' option for compatibility.
Also, this uses 'vite' instead of 'webpack' to avoid circular dependency warning although webpack is fully tested with wasm-bindgen.
I have no idea about it.

## How to install npm packages

```sh
npm install
```

## How to build in **debug** mode

```sh
npm run build
```

## Hot to run vite dev server

```sh
npm start
```

## How to build in **release** mode

```sh
npm run build-release
```

## How to clean up after build or test

```sh
# Remove "dist" and "pkg" directories
npm run clean

# Remove all directories produced by build and test processes
npm run clean-all
```

