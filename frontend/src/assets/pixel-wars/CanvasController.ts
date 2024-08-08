const CANVAS_SCALE = 3;
const DEFAULT_SIZE = 252;

const LEFT_BUTTON = 0;
const MIDDLE_BUTTON = 1;

function numberClamp(num, min, max) {
  return Math.min(Math.max(num, min), max)
}

export const CANVAS_UPDATE = "canvasUpdate";

export default class CanvasElementController {
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  pixels: CanvasPixels;

  private _canvas_zoom = 5;
  public get canvas_zoom() { return this._canvas_zoom }
  public set canvas_zoom(value) { this._canvas_zoom = value; this.emit('canvas_zoom', value) }

  private cursor_position_x = "0px";
  private cursor_position_y = "0px";

  private _canvas_view_translate_x = "0px";
  public get canvas_view_translate_x() { return this._canvas_view_translate_x }
  public set canvas_view_translate_x(value) { this._canvas_view_translate_x = value; this.emit('canvas_view_translate_x', value) }
  private _canvas_view_translate_y = "0px";
  public get canvas_view_translate_y() { return this._canvas_view_translate_y }
  public set canvas_view_translate_y(value) { this._canvas_view_translate_y = value; this.emit('canvas_view_translate_y', value) }

  private _cursor_canvas_x = 0;
  public get cursor_canvas_x() { return this._cursor_canvas_x }
  public set cursor_canvas_x(value) { this._cursor_canvas_x = value; this.emit('cursor_canvas_x', value) }
  private _cursor_canvas_y = 0;
  public get cursor_canvas_y() { return this._cursor_canvas_y }
  public set cursor_canvas_y(value) { this._cursor_canvas_y = value; this.emit('cursor_canvas_y', value) }

  private canvas_can_move = false;
  private canvas_can_place = false;

  private canvas_move_frame_asked = false;
  private canvas_update_frame_asked = false;

  constructor(canvas: HTMLCanvasElement, map_size = DEFAULT_SIZE) {
    this.canvas = canvas;
    this.setSize(map_size);
    this.ctx = this.canvas.getContext('2d');
    this.ctx.imageSmoothingEnabled = false;
    this.ctx.scale(CANVAS_SCALE, CANVAS_SCALE);

    this.updateCanvas();

    this.registerEventListeners();
  }

  public setSize(map_size = DEFAULT_SIZE) {
    this.canvas.width = map_size * CANVAS_SCALE;
    this.canvas.height = map_size * CANVAS_SCALE;
  }

  private registerEventListeners() {
    window.addEventListener("mousedown", (e) => {
      if (e.button == LEFT_BUTTON || e.button == MIDDLE_BUTTON) {
        e.preventDefault();
        this.canvas_can_move = true;
      }
    });

    window.addEventListener("mouseup", () => {
      this.canvas_can_move = false;
    });

    window.addEventListener("wheel", (e) => {
      this.canvas_zoom = numberClamp(this.canvas_zoom + (e.deltaY < 0 ? 0.2 : -0.2), 0.3, 8);
    });

    window.addEventListener("mousemove", (e) => {
      window.requestAnimationFrame(() => {
        this.cursor_position_x = `${Math.floor(e.clientX)}px`;
        this.cursor_position_y = `${Math.floor(e.clientY)}px`;
      });
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
      if (this.canvas_can_place)
        this.canvas_can_place = (e.movementX <= 1 && e.movementX >= -1) || (e.movementY <= 1 && e.movementY >= -1);

      const [x, y] = this.getCursorCanvasPosition(e);
      this.cursor_canvas_x = x;
      this.cursor_canvas_y = y;

      if(this.canvas_can_move && !this.canvas_move_frame_asked) {
        this.canvas_move_frame_asked = true;
        window.requestAnimationFrame(() => {
          const delta_x = ((e.movementX * 5) / this.canvas_zoom) * 0.2;
          const delta_y = ((e.movementY * 5) / this.canvas_zoom) * 0.2;
          this.canvas_view_translate_x = String(parseFloat(this.canvas_view_translate_x) + delta_x + "px");
          this.canvas_view_translate_y = String(parseFloat(this.canvas_view_translate_y) + delta_y + "px");
          this.canvas_move_frame_asked = false;
        });
      }
    });
  }

  private getCursorCanvasPosition(event: MouseEvent): [number, number] {
    const rect = this.canvas.getBoundingClientRect();
    const x = Math.max(Math.ceil((event.clientX - rect.left) / CANVAS_SCALE / this.canvas_zoom) - 1, 0);
    const y = Math.max(Math.ceil((event.clientY - rect.top) / CANVAS_SCALE / this.canvas_zoom) - 1, 0);
    return [ x, y ];
  }

  private placePixel(x: number, y: number) {
    window.dispatchEvent(new CustomEvent("pixelClicked", { detail: { x, y } }));
    console.log(x, y);
  }

  private emit(field: string, value: unknown) {
    window.dispatchEvent(new CustomEvent(CANVAS_UPDATE, { detail: {
      field,
      value
    }}));
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

  private updateCanvas() {
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

export type Color = [number, number, number, number];

export type CanvasPixels = {
  colors: Color[]
  width: number
  height: number
}