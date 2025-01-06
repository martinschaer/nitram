/* @refresh reload */
import { render } from "solid-js/web";

import "./index.css";
import App from "./App.tsx";
import { BackendProvider } from "./BackendContext";

const root = document.getElementById("root");

render(
  () => (
    <BackendProvider publicChildren={<App />}>
      <App />
    </BackendProvider>
  ),
  root!,
);
