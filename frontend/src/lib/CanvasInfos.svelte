<script lang="ts">
    import { CanvasInfoStore } from "../assets/pixel-wars/stores";

    const precision = 1;
    const power_precision = Math.pow(10, precision);
    $: rounded_zoom = String(Math.round($CanvasInfoStore.canvas_zoom*power_precision)/power_precision);
</script>

<div class="card">
    ({$CanvasInfoStore.cursor_canvas_x}, {$CanvasInfoStore.cursor_canvas_y}) {rounded_zoom}x
</div>

<style lang="scss">
    .card {
        position: fixed;
        top: var(--card-spacing);
        right: var(--card-spacing);
        display: flex;
        gap: .25rem;
        font-weight: bold;
        line-height: 1rem;
        font-size: 1rem;
        border-radius: 1.1rem;
    }
</style>