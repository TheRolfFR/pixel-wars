<script lang="ts">
    import { ColorPallete } from "../assets/pixel-wars/canvas";
    import { ColorPickerStore } from "../assets/pixel-wars/stores";
    import { TimeoutStore } from '../assets/pixel-wars/stores';

    let number = 0;

    const changeColor = num => (mouseEvent:MouseEvent) => {
        number = num;
        const element = mouseEvent.target as Element;
        element.classList.add("color-block-active");
        ColorPickerStore.set(num);
    };
</script>
<div id="color-picker" class="card">
    <div id="palette-grid">
        {#each ColorPallete as color, i}
            <button class="color-block" class:white={color[0] == 255 && color[1] == 255 && color[2] == 255} style="background-color: rgb({color[0]},{color[1]},{color[2]});"
                on:click="{changeColor(i)}"
                class:color-block-active="{i === number}">
            </button>
        {/each}
    </div>
    <div id="pixels-left">
        <b>{ $TimeoutStore.remainingPixels }</b> pixels left
    </div>
</div>
<style lang="scss">
    $block_size: 2rem;
    $gap_size: 0.4rem;
    $text_size: $block_size * 2 + $gap_size;
    #color-picker {
        position: fixed;
        top: var(--card-spacing);
        right: var(--card-spacing);
        padding: 0.7rem;
    }

    #palette-grid {
        display: grid;
        grid-template-columns: repeat(2, 8fr);
        gap: $gap_size;
    }

    .color-block {
        height: $block_size;
        width: $block_size;
        border: 0.2rem solid transparent;
        transition: border 0.2s;
        position: relative;
        box-shadow: 0px 1px 2px 0px rgba(0, 0, 0, 0.3);
    }

    #pixels-left {
        width: $text_size;
        margin-top: $gap_size;
        text-align: center;
    }

    .color-block-active{
        border: 0.2rem solid rgba(255,255,255,0.4);
    }
    .color-block-active.white {
        border-color: lightgrey;
    }
</style>