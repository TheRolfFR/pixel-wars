import { writable } from "svelte/store";
import { createStore } from "./utils/createStore";


export const ColorPickerStore = createStore("COLOR_PICKER", 0, (s) => Number.parseInt(s, 10), v => v);

const rawTimeoutStore = writable({
    lastDate: new Date(),
    nextDate: new Date(),
    remainingPixels: null,
    requestingPixels: false,
});
export const TimeoutStore = {
    ...rawTimeoutStore,
    request: async function() {
        // update request status
        rawTimeoutStore.update((v) => ({...v, requestingPixels: true }));

        return fetch(window.location.protocol+"//"+window.location.host+'/api/client/details')
            .then(r => r.json())
            .then(json => {
                console.log(json)
                const obj = {
                    lastDate: new Date(json.lastTimestamp * 1000),
                    nextDate: new Date(json.nextTimestamp * 1000),
                    remainingPixels: json.remainingPixels,
                    requestingPixels: false,
                };
                // update data and clear request status
                rawTimeoutStore.set(obj);

                const restart_duration = obj.nextDate.getTime() - new Date().getTime();
                if(restart_duration > 0)
                {
                    setTimeout(() => {
                        this.request();
                    }, restart_duration + 10);
                }

                return obj;
            })
            .finally(() => {
                // update request status
                rawTimeoutStore.update((v) => ({...v, requestingPixels: false }));
            });
    }
}

export const CanvasInfoStore = writable({
    canvas_zoom: 5,

    canvas_view_translate_x: "0px",
    canvas_view_translate_y: "0px",

    cursor_canvas_x: 0,
    cursor_canvas_y: 0,

    height: 0,
    width: 0,
})

export const CanvasPaletteStore = writable([])

export const OnlineCountStore = writable(0);
