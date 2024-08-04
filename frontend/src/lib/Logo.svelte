<script lang="ts">
    import logo from '../assets/logo.png';
    import logoText from '../assets/logo-text.png';
</script>

<div id="logo-card" class="card">
    <img src={logo} alt="rp" id="logo" />
    <img src={logoText} alt="rs/place" id="logo-text" />
</div>

<style lang="scss">
    #logo-card {
        position: fixed;
        top: var(--card-spacing);
        left: var(--card-spacing);
        display: flex;
        align-items: center;

        #logo, #logo-text {
            height: 2rem;
        }
        #logo {
            height: 2rem;
            margin-right: 1.2rem;
        }
    }
</style>
