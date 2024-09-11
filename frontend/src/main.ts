import './app.scss'
import App from './pages/Home.svelte'

const target = document.getElementById('app') as Element;

if (import.meta.env.DEV && !(target instanceof HTMLElement)) {
  throw new Error(
    'Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got mispelled?',
  );
}

// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore svelte-check found no error but ts did
const app = new App({
  target,
});

export default app;

