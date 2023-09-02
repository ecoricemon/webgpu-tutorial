import { run, print_self, set_camera } from "../pkg/wasm-index.js";
import GUIHelper from "./GUIHelper.js";

// Run wasm
await run();
addEventListener("keydown", (ev) => {
  print_self();
})

// Camera eye and center contollers
const camera_eye = {
  x: 0.0,
  y: 0.0,
  z: 1.0
};
const camera_center = {
  x: 0.0,
  y: 0.0,
  z: 0.0
};

const _set_camera = () => {
  set_camera(
    camera_eye.x, camera_eye.y, camera_eye.z,
    camera_center.x, camera_center.y, camera_center.z
  );
};

_set_camera();
GUIHelper
  .select('camera', 'eye position')
  .add(camera_eye, "x", -2, 2)
  .onChange(_set_camera)
  .add(camera_eye, "y", -2, 2)
  .onChange(_set_camera)
  .add(camera_eye, "z", 0.5, 2)
  .onChange(_set_camera)
  ;
GUIHelper
  .select('camera', 'center position')
  .add(camera_center, "x", -2, 2)
  .onChange(_set_camera)
  .add(camera_center, "y", -2, 2)
  .onChange(_set_camera)
  .add(camera_center, "z", -0.5, 2)
  .onChange(_set_camera)
  ;
GUIHelper
  .closeAll();