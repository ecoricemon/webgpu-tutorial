import { run, print_self, set_camera } from "../pkg/wasm-index.js";
import GUIHelper from "./GUIHelper.js";

// Run wasm
await run();
addEventListener("keydown", (ev) => {
  print_self();
})

// Camera and target position contollers
const camera_position = {
  x: 0.0,
  y: 0.0,
  z: 1.0
};
const camera_look_at = {
  x: 0.0,
  y: 0.0,
  z: 0.0
};

const _set_camera = () => {
  set_camera(
    camera_position.x, camera_position.y, camera_position.z,
    camera_look_at.x, camera_look_at.y, camera_look_at.z
  );
};

_set_camera();
GUIHelper
  .select('camera', 'position')
  .add(camera_position, "x", -2, 2)
  .onChange(_set_camera)
  .add(camera_position, "y", -2, 2)
  .onChange(_set_camera)
  .add(camera_position, "z", 0.5, 2)
  .onChange(_set_camera)
  ;
GUIHelper
  .select('camera', 'looking at')
  .add(camera_look_at, "x", -2, 2)
  .onChange(_set_camera)
  .add(camera_look_at, "y", -2, 2)
  .onChange(_set_camera)
  .add(camera_look_at, "z", -0.5, 2)
  .onChange(_set_camera)
  ;
GUIHelper
  .closeAll();