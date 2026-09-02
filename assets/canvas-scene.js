// Base class for info-dense "card" scenes (World Cup tables, fixtures, bracket).
//
// Unlike the shader/sim scenes, these are drawn with the 2D canvas API — far
// easier for typography, flags and tables. A scene subclasses CanvasScene and
// implements draw(ctx, W, H, t, dt); the base owns an offscreen canvas sized to
// the supersampled render resolution (w*dpr, matching the shader passes so the
// composite's downsample gives the same free AA) and exposes it to the
// SceneManager as a THREE.CanvasTexture via the standard `.texture` getter.
//
// Pass interface implemented here: .texture, setSize, frame, enter, exitReady,
// dispose — same contract the manager drives for every scene.

import * as THREE from "three";
import { breath as sharedBreath } from "../../breath.js";

export class CanvasScene {
  constructor(ctx) {
    this.duration = (ctx && ctx.sceneDuration) || 90;
    this.t = 0;
    this.breath = sharedBreath;
    this._w = 1;
    this._h = 1;
    this.canvas = document.createElement("canvas");
    this.canvas.width = 1;
    this.canvas.height = 1;
    this.c2d = this.canvas.getContext("2d");
    this.tex = new THREE.CanvasTexture(this.canvas);
    this.tex.minFilter = THREE.LinearFilter;
    this.tex.magFilter = THREE.LinearFilter;
    this.tex.generateMipmaps = false;
    // Treat the canvas as the scene's linear-ish art buffer: skip three's auto
    // sRGB decode so the composite's gamma lift behaves like it does for the
    // other passes (colors are tuned against that lift, not nominal sRGB).
    this.tex.colorSpace = THREE.NoColorSpace;
  }

  get texture() {
    return this.tex;
  }

  setSize(w, h, dpr) {
    const rw = Math.max(1, Math.floor(w * dpr));
    const rh = Math.max(1, Math.floor(h * dpr));
    if (rw === this._w && rh === this._h) return;
    this._w = rw;
    this._h = rh;
    this.canvas.width = rw;
    this.canvas.height = rh;
    this._dirty = true;
  }

  // Called every frame by the manager. Subclasses that animate can override
  // shouldRedraw(); the default redraws every frame (cheap at these sizes and
  // lets live clocks / pulses move).
  // The shared body clock every board's ambient motion rides (see breath.js).
  // The manager hands it down each frame; falling back to the singleton means a
  // board drawn outside the manager (a test, a one-off harness) still gets a
  // valid — if unadvanced — breath rather than a null to guard against.
  frame(_renderer, t, dt, breath) {
    this.t = t;
    this.dt = dt || 0;
    this.breath = breath || sharedBreath;
    this.draw(this.c2d, this._w, this._h, t, this.dt);
    this.tex.needsUpdate = true;
  }

  // Subclasses implement.
  draw(_ctx, _w, _h, _t, _dt) {}

  enter() {
    this._dirty = true;
  }

  dispose() {
    this.tex.dispose();
  }
}
