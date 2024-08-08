<script lang="ts">
    import logoText from '../assets/logo-text.png';
    import { OnlineCountStore } from '../assets/pixel-wars/stores';

    const formatCount = n => {
        if (n < 1e3) return n;
        if (n >= 1e3 && n < 1e6) return +(n / 1e3).toFixed(1) + "K";
        if (n >= 1e6 && n < 1e9) return +(n / 1e6).toFixed(1) + "M";
        if (n >= 1e9 && n < 1e12) return +(n / 1e9).toFixed(1) + "B";
        if (n >= 1e12) return +(n / 1e12).toFixed(1) + "T";
    };

    $: onlineCount = formatCount($OnlineCountStore);
</script>

<div id="topleft">
    <div id="logo-card" class="card">
        <img src={logoText} alt="rs/place" id="logo-text" />
    </div>
    {#if $OnlineCountStore > 0}
        <div id="online"><span>{onlineCount}</span><span id="onlinedot" /></div>
    {/if}
</div>

<style lang="scss">
    #topleft {
        position: fixed;
        top: var(--card-spacing);
        left: var(--card-spacing);
        display: flex;
        align-items: center;
        z-index: 1000;
        gap: var(--card-spacing);
    }
    #logo-card {
        #logo-text {
            height: 2rem;
            float: left;
        }
        border-radius: 1.6rem;
    }
    #online {
        text-shadow: 0 0 3px #000000;
        & > * {
            display: inline-block;
            vertical-align: middle;
        }
        #onlinedot {
            height: 0.5rem;
            width: 0.5rem;
            margin-left: 0.2rem;
            border-radius: 50%;
            background: #38F286;
        }
    }
</style>
