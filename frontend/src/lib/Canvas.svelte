<script lang="ts">
  import { initialLoad } from '../assets/pixel-wars/canvas';
  import CanvasElementController from '../assets/pixel-wars/CanvasController';
  import { CanvasInfoStore } from '../assets/pixel-wars/stores';
  import SubscriptionController from '../assets/pixel-wars/SubscriptionController';

  let isProduction = import.meta.env.MODE === 'production';
  // @ts-ignore
  if(!isProduction && Window.prototype._addEventListener === undefined) {
    // @ts-ignore
    Window.prototype._addEventListener = Window.prototype.addEventListener;
      Window.prototype.addEventListener = function(event, func, passive) {
        // @ts-ignore
        const eventdate = window.eventdate;
        if (passive==undefined) passive=false;
        this._addEventListener(event,(...args) => {
          // @ts-ignore
          if(window.eventdate === eventdate) {
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
      // @ts-ignore
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
