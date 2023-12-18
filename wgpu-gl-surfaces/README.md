# Trouble

During use of wgpu, if I make surfaces like below, wgpu will draw something onto the B.

Pseudo code
let surface_a = instance.create_surface(canvas_a); 
let surface_b = instance.create_surface(canvas_b); 
let adapter = instance.request_adapter();
let (device, queue) = adapter.request_device();
let surface_c = instance.create_surface(canvas_c);
...
let texture = surface_c.get_current_texture(); << Intended to render onto canvac_c.
let view = texture.create_view();
render to view << rendered onto surface canvas_b.

Analysis

In WebGL, each canvas has render context and 
it will be kept in wgpu::Instance when we're creating wgpu::Surface.
It'll be overwritten when we make multiple surfaces.
So that wgpu::Instance has render context of canvas_b.
wgpu::Device will use the context, 
not the thing of canvas_c even if we pass the view from canvas_c.


## How to install npm packages

```sh
npm install
```

## How to build in **debug** mode

```sh
npm run build
```

## Hot to run webpack dev server

```sh
npm start
```

## How to clean up after build or test

```sh
# Remove "dist" and "pkg" directories
npm run clean

# Remove all directories produced by build and test processes
npm run clean-all
```
