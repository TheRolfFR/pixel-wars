import { writable } from "svelte/store";
import { createStore } from "./utils/createStore";


export const ColorPickerStore = createStore("COLOR_PICKER", 0, (s) => Number.parseInt(s, 10), v => v);

export const TimeoutStore = writable({timeout: new Date(), remainingPixels: 0})

export const CanvasInfoStoreDefault = {
    canvas_zoom: 5,

    canvas_view_translate_x: "0px",
    canvas_view_translate_y: "0px",

    cursor_canvas_x: 0,
    cursor_canvas_y: 0,
};

export const CanvasInfoStore = writable(CanvasInfoStoreDefault)

export const OnlineCountStore = writable(0);
