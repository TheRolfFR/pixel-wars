<script lang="ts">
  import { TimeoutStore } from '../assets/pixel-wars/stores';
  import timeFormat from '../assets/pixel-wars/utils/timeFormat';

  let diffTime: number = 0;

  TimeoutStore.subscribe(async (timeout) => {
    if(timeout.remainingPixels === null) return;
    if(timeout.remainingPixels > 0) return;

    const timerDuration = timeout.nextDate.getTime() - new Date().getTime();
    if(timerDuration > 0) {
      // keep this line to show on first second
      diffTime = (timeout.nextDate.getTime() - new Date().getTime());

      // then start the timer
      let timeoutHandle = setInterval(() => {
        diffTime = Math.max((timeout.nextDate.getTime() - new Date().getTime()), 0);
      }, 1000);
      setTimeout(() => {
        clearInterval(timeoutHandle);
      }, diffTime + 1000);
    }
  });

  $: secondsLeft = Math.round(diffTime / 1000);
  $: showTimeout = $TimeoutStore.remainingPixels === 0;
  $: message = secondsLeft === 0 ? `More pixels soon...` : `More pixels in ${timeFormat(secondsLeft)}`;
</script>

<div id="timeout-message" class:active={showTimeout}>
  <p>You changed too many pixels!</p>
  <p>{message}</p>
</div>

<style lang="scss">
  #timeout-message {
    padding: 0.7rem 0.7rem 0;
    display: none;
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    border-radius: 0.6rem;
    background-color: rgba(255,255,255,0.6);
    font-weight: 600;
    flex-direction: column;
    justify-content: space-evenly;

    p {
      margin: 0;
    }
  }

  .active {
    display: flex !important;
  }
</style>
