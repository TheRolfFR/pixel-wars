<script lang="ts">
  import { initialLoad } from '../assets/pixel-wars/canvas';
  import CanvasElementController from '../assets/pixel-wars/CanvasController';
  import { CanvasInfoStore } from '../assets/pixel-wars/stores';
  import SubscriptionController from '../assets/pixel-wars/SubscriptionController';

  let isProduction = import.meta.env.MODE === 'production';
  // @ts-expect-error Dev override for svelte multiplying window events
  if(!isProduction && Window.prototype._addEventListener === undefined) {
    // @ts-expect-error Dev override for svelte multiplying window events
    Window.prototype._addEventListener = Window.prototype.addEventListener;
      Window.prototype.addEventListener = function(
        event: string,
        // eslint-disable-next-line no-undef
        func: EventListenerOrEventListenerObject,
        // eslint-disable-next-line no-undef
        passive?: boolean | AddEventListenerOptions
      ) {
        // @ts-expect-error Custom attribute
        const eventdate = window.eventdate;
        if (passive==undefined) passive=false;
        // @ts-expect-error Custom method
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        this._addEventListener(event,(...args: Array<any>) => {
          // @ts-expect-error Custom atribute eventdate
          if(window.eventdate === eventdate) {
            // eslint-disable-next-line @typescript-eslint/ban-ts-comment
            // @ts-ignore Propagate everything
            func(...args)
          }
        },passive);
    };
  }

  let canvasController: CanvasElementController;
  let subscriptionController: SubscriptionController;

  $: styles = Object.entries({
    transform: `scale(${$CanvasInfoStore.canvas_zoom}) translate(${$CanvasInfoStore.canvas_view_translate_x }, ${$CanvasInfoStore.canvas_view_translate_y })`
  }).map(([key, value]) => `${key}: ${value}`).join("; ");

  const init = (canvasElement: HTMLCanvasElement) => {
    (async () => {
      // @ts-expect-error Custom attribute
      window.eventdate = Date.now();

      canvasController = new CanvasElementController(canvasElement);
      subscriptionController = new SubscriptionController(canvasController);

      await subscriptionController.initConnection();
      await initialLoad(canvasController);
    })();
  }
</script>
<div id="canvas-container">
  <canvas id="canvas-square" style={styles} use:init />
</div>
<style>
  canvas {
    image-rendering: pixelated;
  }
</style>
