<script lang="ts">
    import logoText from '../assets/logo-text.png';
    import { OnlineCountStore, CanvasInfoStore } from "../assets/pixel-wars/stores";
    import Icon from './Icon.svelte';

    const formatCount = n => {
        if (n < 1e3) return n;
        if (n >= 1e3 && n < 1e6) return +(n / 1e3).toFixed(1) + "K";
        if (n >= 1e6 && n < 1e9) return +(n / 1e6).toFixed(1) + "M";
        if (n >= 1e9 && n < 1e12) return +(n / 1e9).toFixed(1) + "B";
        if (n >= 1e12) return +(n / 1e12).toFixed(1) + "T";
    };

    const is_touch_device = 'ontouchstart' in window;

    $: online_count = formatCount($OnlineCountStore);

    const precision = 1;
    const power_precision = Math.pow(10, precision);
    $: rounded_zoom = String(Math.round($CanvasInfoStore.canvas_zoom*power_precision)/power_precision);
</script>

<div id="topleft">
    <div id="logo-card" class="card">
        <img src={logoText} alt="rs/place" id="logo-text" />
    </div>

    <div id="tips">
        <p>
          <Icon variant="users" />
          <span>{online_count}</span>
        </p>
        <p>
          <Icon variant="edit" />
          <span>{ is_touch_device ? "Tap" : "Left Click" }</span>
        </p>
        <p>
          <Icon variant="move" />
          <span>{ is_touch_device ? "Drag & Pinch" : "Click & Drag" }</span>
        </p>
      </div>
</div>

<div id="topright" class="card">
    {#if !is_touch_device}
        ({$CanvasInfoStore.cursor_canvas_x}, {$CanvasInfoStore.cursor_canvas_y}) 
    {/if}{rounded_zoom}x
</div>


<style lang="scss">
    #topleft {
        position: fixed;
        top: var(--card-spacing);
        left: var(--card-spacing);
        display: flex;
        align-items: center;
        gap: var(--card-spacing);
    }
    #logo-card {
        border-radius: 1.6rem;

        #logo-text {
            height: 2rem;
            float: left;
        }
    }

    #tips {
        position: absolute;
        left: 0;
        top: 100%;
        display: flex;
        flex-direction: column;
        gap: .5rem;
        font-weight: 500;
        margin-left: var(--card-spacing);
        margin-top: var(--card-spacing);

        p {
            display: flex;
            justify-content: start;
            justify-items: center;
            gap: 0.5rem;
            margin: 0;

            &:global(> *:first-child) {
                width: 1rem;
            }
        }
    }

    #topright {
        position: fixed;
        top: var(--card-spacing);
        right: var(--card-spacing);
        font-weight: bold;
        line-height: 1rem;
        font-size: 1rem;
        border-radius: 1.1rem;
    }
</style>
