<script lang="ts">
  import { initialLoad } from '../assets/pixel-wars/canvas';
  import CanvasElementController, { CANVAS_UPDATE } from '../assets/pixel-wars/CanvasController';
    import { CanvasInfoStore, CanvasInfoStoreDefault } from '../assets/pixel-wars/stores';
  import SubscriptionController from '../assets/pixel-wars/SubscriptionController';
  import CanvasOverlay from './CanvasOverlay.svelte';

  let canvasElement: HTMLCanvasElement;
  let canvasController: CanvasElementController;
  let subscriptionController: SubscriptionController;

  let dyn = CanvasInfoStoreDefault;

  $: styles = Object.entries({
    transform: `scale(${dyn.canvas_zoom}) translate(${dyn.canvas_view_translate_x }, ${dyn.canvas_view_translate_y })`
  }).map(([key, value]) => `${key}: ${value}`).join(";");

  window.addEventListener(CANVAS_UPDATE, (e: CustomEvent) => {
    let {field, value}: {
      field: string,
      value: unknown
    } = e.detail;
    dyn[field] = value;
    CanvasInfoStore.set(dyn)
  })

  window.addEventListener('load', async () => {
    canvasController = new CanvasElementController(canvasElement);
    subscriptionController = new SubscriptionController(canvasController);
    await subscriptionController.initConnection();
    await initialLoad(canvasController);
  });
</script>
<div id="canvas-container">
  <canvas id="canvas-square" bind:this={canvasElement} style={styles} />
  <CanvasOverlay />
</div>
<style>
  canvas {
    image-rendering: pixelated;
  }
</style>
