import { mount } from 'svelte';
import App from './App.svelte';
import { getLocale } from './lib/paraglide/runtime.js';

document.documentElement.lang = getLocale();

const app = mount(App, { target: document.getElementById('app')! });

export default app;
