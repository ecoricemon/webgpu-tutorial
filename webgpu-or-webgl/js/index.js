if (navigator.gpu) {
  // Loads wasm built on WebGPU.
  import("../pkg_gpu/wasm-index.js").then((wasm) => {
    run(wasm);
  });
} else {
  // Loads wasm built on WebGL.
  import("../pkg_gl/wasm-index.js").then((wasm) => {
    run(wasm);
  });
}

async function run(wasm) {
  // Runs wasm and waits for it.
  await wasm.run();

  // getContext() will return null if it's not matched with the thing wasm set above.
  // See https://developer.mozilla.org/en-US/docs/Web/API/HTMLCanvasElement/getContext
  const canvas = document.getElementById("canvas0");
  let backend = undefined;
  if (canvas.getContext("webgl") !== null) {
    backend = "WebGL";
  } else if (canvas.getContext("webgl2") !== null) {
    backend = "WebGL2";
  } else if (canvas.getContext("webgpu") !== null) {
    backend = "WebGPU";
  } 

  if (backend !== undefined) {
    const element = document.getElementById("backend");
    element.innerHTML = "Running " + backend;
  }
}
