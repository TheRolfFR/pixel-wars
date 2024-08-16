<script lang="ts">
  import { get } from 'svelte/store';

  import { initialLoad } from '../assets/pixel-wars/canvas';
  import { CanvasElementController, CANVAS_UPDATE } from '../assets/pixel-wars/CanvasController';
  import { CanvasInfoStore } from '../assets/pixel-wars/stores';
  import SubscriptionController from '../assets/pixel-wars/SubscriptionController';

  let canvasController: CanvasElementController;
  let subscriptionController: SubscriptionController;

  $: styles = Object.entries({
    transform: `scale(${$CanvasInfoStore.canvas_zoom}) translate(${$CanvasInfoStore.canvas_view_translate_x }, ${$CanvasInfoStore.canvas_view_translate_y })`
  }).map(([key, value]) => `${key}: ${value}`).join("; ");

  const init = (canvasElement: HTMLCanvasElement) => {
    (async () => {
      // @ts-ignore
      window.eventdate = Date.now();

      window.addEventListener(CANVAS_UPDATE, (e: CustomEvent) => {
        const dyn = get(CanvasInfoStore);
        let {field, value}: {
          field: string,
          value: unknown
        } = e.detail;
        dyn[field] = value;
        CanvasInfoStore.set(dyn)
      });

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
