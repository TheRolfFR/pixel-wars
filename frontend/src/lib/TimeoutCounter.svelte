<script lang="ts">
  import { TimeoutStore } from '../assets/pixel-wars/stores';

  let diffTime: number = 0;

  TimeoutStore.subscribe(async (timeout) => {
    console.log(timeout);
    if (timeout.remainingPixels == 0) {
      let query = await fetchLastTimeout();
      let nextDate = new Date((query.timeout + 60) * 1000);
      if (query.remainingPixels != 0) {
        TimeoutStore.set({
          timeout: new Date(query.timeout * 1000),
          remainingPixels: query.remainingPixels
        });
        return;
      }
      let timeoutHandle = setInterval(()=>{
        diffTime = (nextDate.getTime() - new Date().getTime())
      }, 1000)
      await new Promise((r) => setTimeout(r, nextDate.getTime() - new Date().getTime()));
      query = await fetchLastTimeout();
      while (query.remainingPixels == 0) {
        await new Promise((r) => setTimeout(r, 1000));
        query = await fetchLastTimeout();
      }
      clearTimeout(timeoutHandle);
      TimeoutStore.set({
        timeout: new Date(query.timeout * 1000),
        remainingPixels: query.remainingPixels
      });
    }
  });

  $: secondsLeft = Math.round(diffTime / 1000);

  async function fetchLastTimeout(): Promise<{ timeout: number; remainingPixels: number }> {
    const query = await fetch(window.location.protocol+"//"+window.location.host+'/pixelwars/api/client/details');
    if (query.status != 200) {
      return {timeout: -1, remainingPixels: -1};
    }
    const json = await query.json();
    return { timeout: json.lastTimestamp, remainingPixels: json.remainingPixels };
  }

  window.addEventListener('sessionLoaded', async (ev) => {
    const query = await fetchLastTimeout();
    if(query.remainingPixels == -1){
      TimeoutStore.set({
        timeout: new Date(0),
        remainingPixels: 9
      });
      return;
    }
    TimeoutStore.set({
      timeout: new Date(query.timeout * 1000),
      remainingPixels: query.remainingPixels
    });
  });

  function timeFormat(secs: number): String {
    let hours = Math.floor(secs / 3600);
    let minutes = Math.floor((secs % 3600) / 60);
    let seconds = secs % 60;
    let result = ``;
    if(hours > 0) result += `${hours}h`;
    if(minutes > 0) result += `${minutes}m`;

    result += `${seconds}s`;

    return result;
  }

  $: show_counter = $TimeoutStore.remainingPixels == 0 && secondsLeft > 0;
</script>

<div id="timeout-message" class:active={show_counter}>
  <p>You changed too many pixels!</p>
  <p>More pixels in {timeFormat(secondsLeft)}</p>
</div>

<style>
  #timeout-message {
    display: none;
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    border-radius: 0.6rem;
    background-color: rgba(255,255,255,0.6);
    font-weight: 600;
  }

  .active {
    display: block !important;
  }
</style>
