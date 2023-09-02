import { GUI } from 'lil-gui';

export default class GUIHelper { 
  static #getInstance() {
    if (!GUIHelper.instance) {
      GUIHelper.instance = this;
      this.gui = new GUI();
      this.folders = undefined;
      this.controllers = [];
    }
    return GUIHelper.instance;
  }

  constructor() {
    return GUIHelper.#getInstance();
  }

  /**
   * Add folders.
   * It will generate folders if it doesn't exist.
   * @param  {...string} folders Folder names.
   * @returns {GUIHelper} Instance for method chaining.
   * @example
   * // Select or Generate folder1.
   * GUIHelper.select('folder1');  
   * // Select or Generate folder2 and folder2-1 within folder2.
   * GUIHelper.select('folder2', 'folder2-1');
   */
  static select(...folders) {
    const instance = GUIHelper.#getInstance();
    instance.folders = folders;
    return instance;
  }

  /**
   * Add a controller.  
   * @param {*} obj An object to be controlled.
   * @param {string} prop A property in the object to be controlled.
   * @param {*} min The minimum value of the property.
   * @param {*} max The maximum value of the property.
   * @param {*} step Step of the property.
   * @param {boolean} r2d Flag to convert radian to degree.
   * @returns {GUIHelper} Instance for method chaining.
   * @example
   * const values = {
   *   x: 0,  
   *   y: 1,
   *   z: .5
   * }  
   * GUIHelper.add(values, 'x');  
   * GUIHelper.add(values, 'y', 0, 100);
   * GUIHelper.add(values, 'z', -10, 10, .1);
   */
  static add(obj, prop, min = undefined, max = undefined, step = undefined, r2d = false) {
    class RadDegConverter {
      constructor(obj, prop) {
        this.obj = obj;
        this.prop = prop;
      }
      get v() {
        return this.obj[this.prop] * 180 / Math.PI;
      }
      set v(v) {
        this.obj[this.prop] = v * Math.PI / 180;
      }
    }

    const instance = GUIHelper.#getInstance();
    instance.controllers.push(
      !r2d ? 
        GUIHelper.#getContainer().add(obj, prop, min, max, step) :
        GUIHelper.#getContainer().add(new RadDegConverter(obj, prop), 'v', min, max, step).name(prop)
    );
    return instance;
  }

  /**
   * Add x, y, and z controller for a vecotr.
   * @param {*} obj An object to be controlled.
   * @param {string} prop Vector name.
   * @param {*} min The minimum value of the property.
   * @param {*} max The maximum value of the property.
   * @param {*} step Step of the property.
   * @param {boolean} r2d Flag to convert radian to degree.
   * @returns {GUIHelper} Instance for method chaining.
   * @example
   * const mesh = new THREE.Mesh(geo, mat);
   * GUIHelper.addVector3(mesh, 'position', -1, 1, .1);
   */
  static addVector3(obj, prop, min = undefined, max = undefined, step = undefined, r2d = false) {
    const instance = GUIHelper.#getInstance();
    this.add(obj[prop], 'x', min, max, step, r2d);
    this.add(obj[prop], 'y', min, max, step, r2d);
    this.add(obj[prop], 'z', min, max, step, r2d);
    return instance;
  }

  /**
   * Add a color controller.
   * @param {*} obj An object to be controlled.
   * @param {string} prop Color property name.
   * @returns {GUIHelper} Instance for method chaining.
   */
  static addColor(obj, prop = 'color') {
    const instance = GUIHelper.#getInstance();
    instance.controllers.push(GUIHelper.#getContainer().addColor(obj, prop));
    return instance;
  }

  /**
   * Set the names of the last controllers.
   * @param {...string} names
   * @returns {GUIHelper} Instance for method chaining.
   * @example
   * GUIHelper
   *   .add(...)
   *   .name('test')
   */
  static name(...names) {
    const instance = GUIHelper.#getInstance();
    const controllers = instance.controllers;
    for (let i = names.length - 1, j = controllers.length - 1; i >= 0 && j >= 0; --i, --j)
      controllers[j].name(names[i]);
    return instance;
  }

  /**
   * Add a callback for the last controller.
   * @param {function} cb 
   * @returns {GUIHelper} Instance for method chaining.
   * @example
   * GUIHelper
   *   .add(...)
   *   .onChange(callback);
   */
  static onChange(cb) {
    const instance = GUIHelper.#getInstance();
    const controllers = instance.controllers;
    if (controllers.length)
      controllers[controllers.length-1].onChange(cb);
    return instance;
  }

  /**
   * Add a callback for the all controllers.
   * @param {function} cb 
   * @returns {GUIHelper} Instance for method chaining.
   * @example
   * GUIHelper
   *   .add(...)
   *   .add(...)
   *   .onChange(callback);
   */
  static onChangeAll(cb) {
    const instance = GUIHelper.#getInstance();
    instance.controllers.forEach(c => c.onChange(cb));
    return instance;
  }

  /**
   * Close all folders
   * @returns {GUIHelper} Instance for method chaining.
   */
  static closeAll() {
    const instance = GUIHelper.#getInstance();
    const helper = (gui) => {
      if (!(gui instanceof GUI))
        return;
      gui.close();
      gui.children.forEach(child => helper(child));
    };
    helper(instance.gui);
    return instance;
  }

  /**
   * Hide control
   */
  static hide() {
    GUIHelper.#getInstance().gui.hide();
  }

  static #getContainer() {
    const instance = GUIHelper.#getInstance();
    const gui = instance.gui;
    const folders = instance.folders;

    const helper = (gui, folders) => {
      if (!folders || !folders.length)
        return gui;
      let child = gui.children.find(child => child._title === folders[0]);
      if (!child)
        child = gui.addFolder(folders[0]);
      return helper(child, folders.slice(1));
    };
    return helper(gui, folders);
  }
}