import { get } from "svelte/store";
import { CanvasInfoStore } from "./stores";

const CANVAS_SCALE = 3;
const DEFAULT_SIZE = 252;

const LEFT_BUTTON = 0;
const MIDDLE_BUTTON = 1;

function numberClamp(num, min, max) {
  return Math.min(Math.max(num, min), max)
}

export class CanvasElementController {
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  pixels: CanvasPixels;

  private canvas_can_move = false;
  private canvas_can_place = false;

  private canvas_move_frame_asked = false;
  private canvas_update_frame_asked = false;

  private mobile_is_zooming = false;
  private mobile_pinch_length_last = 0;
  private mobile_pan_position_last = [0,0];

  constructor(canvas: HTMLCanvasElement, map_size = DEFAULT_SIZE) {
    this.canvas = canvas;
    this.setSize(map_size);

    this.updateCanvas();

    this.registerEventListeners();
  }

  public setSize(width = DEFAULT_SIZE, height = undefined) {
    if(height === undefined) height = width;

    this.canvas.width = width * CANVAS_SCALE;
    this.canvas.height = height * CANVAS_SCALE;
    this.ctx = this.canvas.getContext('2d');
    this.ctx.imageSmoothingEnabled = false;
    this.ctx.scale(CANVAS_SCALE, CANVAS_SCALE);
  }

  private registerEventListeners() {
    window.addEventListener("mousedown", (e) => {
      if (e.button == LEFT_BUTTON || e.button == MIDDLE_BUTTON) {
        e.preventDefault();
        this.canvas_can_move = true;
      }
    });

    window.addEventListener("touchstart", (touch) => {
      if (touch.touches.length == 1) {
        touch.preventDefault();
        this.canvas_can_move = true;
      }
    });

    window.addEventListener("mouseup", () => {
      this.canvas_can_move = false;
    });

    window.addEventListener("wheel", (e) => {
      CanvasInfoStore.update((v) => ({ ...v, canvas_zoom: numberClamp(v.canvas_zoom + (e.deltaY < 0 ? 0.2 : -0.2), 0.3, 8) }));
    });

    this.canvas.addEventListener("mousedown", (e: MouseEvent) => {
      if (e.button == LEFT_BUTTON) {
        this.canvas_can_place = true;
      }
    });
    this.canvas.addEventListener("mouseup", (e: MouseEvent) => {
      if (e.button != LEFT_BUTTON || !this.canvas_can_place) return;

      const xy = this.getCursorCanvasPosition(e);
      this.placePixel(...xy);
    });
    this.canvas.addEventListener("mousemove", (e: MouseEvent) => {
      const movementX = e.movementX;
      const movementY = e.movementY;
      if (this.canvas_can_place)
        this.canvas_can_place = (movementX <= 1 && movementX >= -1) || (movementY <= 1 && movementY >= -1);

      const [x, y] = this.getCursorCanvasPosition(e);
      CanvasInfoStore.update(v => ({
        ...v,
        cursor_canvas_x: x,
        cursor_canvas_y: y
      }));

      if(this.canvas_can_move && !this.canvas_move_frame_asked) {
        this.canvas_move_frame_asked = true;
        window.requestAnimationFrame(() => {
          const delta_x = ((movementX * 5) / get(CanvasInfoStore).canvas_zoom) * 0.2;
          const delta_y = ((movementY * 5) / get(CanvasInfoStore).canvas_zoom) * 0.2;
          CanvasInfoStore.update(v => ({
            ...v,
            canvas_view_translate_x: String(parseFloat(v.canvas_view_translate_x) + delta_x + "px"),
            canvas_view_translate_y: String(parseFloat(v.canvas_view_translate_y) + delta_y + "px")
          }));
          this.canvas_move_frame_asked = false;
        });
      }
    });

    // touch equivalent
    this.canvas.addEventListener("touchstart", (touch) => {
      touch.preventDefault();
      if (touch.touches.length == 2) {
        this.mobile_pinch_length_last = Math.hypot(
          touch.touches[0].clientX - touch.touches[1].clientX,
          touch.touches[0].clientY - touch.touches[1].clientY
        );
        this.mobile_is_zooming = true;
      } else if (touch.touches.length == 1) {
        this.mobile_pan_position_last = [
          touch.touches[0].clientX,
          touch.touches[0].clientY,
        ];
        this.canvas_can_move = true;
        this.canvas_can_place = true;
      }
    });
    this.canvas.addEventListener("touchmove", (touch) => {
      if (touch.touches.length == 2 && this.mobile_is_zooming) {
        const mobile_pinch_length = Math.hypot(
          touch.touches[0].clientX - touch.touches[1].clientX,
          touch.touches[0].clientY - touch.touches[1].clientY
        );
        if (mobile_pinch_length - this.mobile_pinch_length_last == 0) return;

        const scalediff = (mobile_pinch_length - this.mobile_pinch_length_last) * 0.01;
        CanvasInfoStore.update(v => ({...v, canvas_zoom: numberClamp(v.canvas_zoom + scalediff, 0.3, 8) }));
        this.mobile_pinch_length_last = mobile_pinch_length;
      } else if (touch.touches.length == 1) {
        const movementX = touch.touches[0].clientX - this.mobile_pan_position_last[0];
        const movementY = touch.touches[0].clientY - this.mobile_pan_position_last[1];
        if (this.canvas_can_place)
          this.canvas_can_place = (movementX <= 1 && movementX >= -1) || (movementY <= 1 && movementY >= -1);

        this.mobile_pan_position_last[0] = touch.touches[0].clientX;
        this.mobile_pan_position_last[1] = touch.touches[0].clientY;

        if(this.canvas_can_move && !this.canvas_move_frame_asked) {
          this.canvas_move_frame_asked = true;
          window.requestAnimationFrame(() => {
            const delta_x = ((movementX * 5) / get(CanvasInfoStore).canvas_zoom) * 0.2;
            const delta_y = ((movementY * 5) / get(CanvasInfoStore).canvas_zoom) * 0.2;
            CanvasInfoStore.update(v => ({
              ...v,
              canvas_view_translate_x: String(parseFloat(v.canvas_view_translate_x) + delta_x + "px"),
              canvas_view_translate_y: String(parseFloat(v.canvas_view_translate_y) + delta_y + "px")
            }));
            this.canvas_move_frame_asked = false;
          });
        }
      }
    });
    this.canvas.addEventListener("touchend", (touch) => {
      this.canvas_can_move = false;
      if (touch.changedTouches.length == 2 && this.mobile_is_zooming) {
        this.mobile_is_zooming = false;
      } else if (touch.changedTouches.length == 1) {
        if (!this.canvas_can_place) return;

        const xy = this.getCursorCanvasPositionMobile(touch);
        this.placePixel(...xy);
      }
    });
  }

  private getCursorCanvasPosition(event: MouseEvent): [number, number] {
    const rect = this.canvas.getBoundingClientRect();
    const x = Math.max(Math.ceil((event.clientX - rect.left) / CANVAS_SCALE / get(CanvasInfoStore).canvas_zoom) - 1, 0);
    const y = Math.max(Math.ceil((event.clientY - rect.top) / CANVAS_SCALE / get(CanvasInfoStore).canvas_zoom) - 1, 0);
    return [ x, y ];
  }

  private getCursorCanvasPositionMobile(event: TouchEvent): [number, number] {
    const rect = this.canvas.getBoundingClientRect();
    const x = Math.max(Math.ceil((event.changedTouches[0].clientX - rect.left) / CANVAS_SCALE / get(CanvasInfoStore).canvas_zoom) - 1, 0);
    const y = Math.max(Math.ceil((event.changedTouches[0].clientY - rect.top) / CANVAS_SCALE / get(CanvasInfoStore).canvas_zoom) - 1, 0);
    return [ x, y ];
  }

  private placePixel(x: number, y: number) {
    window.dispatchEvent(new CustomEvent("pixelClicked", { detail: { x, y } }));
  }

  putCanvasPixels(canvasPixels: CanvasPixels) {
    this.pixels = canvasPixels;
    this.updateCanvas();
  }

  private putPixel(x: number, y: number, [r, g, b, a]: Color) {
    this.ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${a})`;
    this.ctx.fillRect(x, y, 1, 1);
  }

  putPixelCanvas(x: number, y: number, color: Color) {
    this.pixels.colors[x + (this.pixels.width * y)] = color;
    this.updateCanvas();

    this.canvas_update_frame_asked = true;
    if (this.canvas_update_frame_asked) window.requestAnimationFrame(() => this.updateCanvas())
  }

  getPixelCanvas(x: number, y: number): Color {
    return this.pixels.colors[x + (this.pixels.width * y)];
  }

  updateCanvas() {
    if(this.pixels === undefined) return;

    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    for (let h = 0; h < this.pixels.height; h++) {
      for (let w = 0; w < this.pixels.width; w++) {
        this.putPixel(w, h, this.pixels.colors[w + (this.pixels.width * h)]);
      }
    }

    this.canvas_update_frame_asked = false;
  }
}
export default CanvasElementController;

export type Color = [number, number, number, number];

export type CanvasPixels = {
  colors: Color[]
  width: number
  height: number
}