<script lang="ts">
    import logoText from '../assets/logo-text.png';
</script>

<div id="logo-card" class="card">
    <img src={logoText} alt="rs/place" id="logo-text" />
</div>

<style lang="scss">
    #logo-card {
        position: fixed;
        top: var(--card-spacing);
        left: var(--card-spacing);
        display: flex;
        align-items: center;

        #logo-text {
            height: 2rem;
        }
        border-radius: 1.6rem;
    }
</style>
