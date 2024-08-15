<script lang="ts">
    import { ColorPallete } from "../assets/pixel-wars/canvas";
    import { ColorPickerStore } from "../assets/pixel-wars/stores";
    import { TimeoutStore } from '../assets/pixel-wars/stores';

    const changeColor = num => (mouseEvent:MouseEvent) => {
        const element = mouseEvent.target as Element;
        element.classList.add("color-block-active");
        ColorPickerStore.set(num);
    };
</script>
<div id="color-picker-placement">
    <div id="color-picker" class="card">
        <div id="palette-grid-scroller">
            <div id="palette-grid">
                {#each ColorPallete as color, i}
                    <button class="color-block" class:white={color[0] == 255 && color[1] == 255 && color[2] == 255} style="background-color: rgb({color[0]},{color[1]},{color[2]});"
                        on:click="{changeColor(i)}"
                        class:color-block-active="{i === $ColorPickerStore}">
                    </button>
                {/each}
            </div>
        </div>
        <div class="pixels-left desktop">
            <b>{ $TimeoutStore.remainingPixels }</b>
        </div>
        <div class="pixels-left mobile">
            { $TimeoutStore.remainingPixels } pixels left
        </div>
    </div>
</div>
<style lang="scss">
    $block_size: 2rem;
    $gap_size: 0.4rem;

    #color-picker-placement {
        position: fixed;
        bottom: var(--card-spacing);
        left: var(--card-spacing);
        right: var(--card-spacing);
        text-align: center;
    }
    #color-picker {
        position: absolute;
        left: 50%;
        bottom: 0;
        transform: translateX(-50%);
        padding: 0.7rem;
        margin: 0 auto;
        display: flex;
        align-items: center;
        max-width: 100%;
        gap: $gap_size*2;
    }

    #palette-grid {
        display: grid;
        grid-template-columns: repeat(8, 2fr);
        gap: $gap_size;
    }

    .color-block, .pixels-left b {
        height: $block_size;
        width: $block_size;
        line-height: $block_size;
        box-shadow: 0px 1px 2px 0px rgba(0, 0, 0, 0.3);
    }
    .color-block {
        border: 0.2rem solid transparent;
        transition: border 0.2s;
    }
    .color-block-active{
        border: 0.2rem solid rgba(255,255,255,0.4);
    }
    .color-block-active.white {
        border-color: lightgrey;
    }

    .pixels-left {
        text-align: center;

        b {
            display: inline-block;
            background: black;
            color: white;
            text-align: center;
        }
    }

    @media(max-width: 400px) {
        #color-picker {
            width: 100%;
            display: block;
            padding-left: 0;
            padding-right: 0;
        }
        #palette-grid-scroller {
            padding: $gap_size;
            height: $block_size + 3*$gap_size;
            overflow: scroll hidden;
        }
        #palette-grid {
            display: block;
            width: 16*$block_size + 17*$gap_size;
            gap: $gap_size;

            & > * {
                display: inline-block;
                margin-left: $gap_size;

                &:first-child {
                    margin-left: 0;
                }
                &:left-child {
                    margin-right: $gap_size;
                }
            }
        }
        .pixels-left {
            margin-top: $gap_size;
        }
    }
</style>