import App from "./App.svelte";
import { mount } from "svelte";

const target = document.getElementById("app")!;
target.innerHTML = "";

const app = mount(App, { target });

export default app;
