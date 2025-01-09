/* @refresh reload */
import { render } from "solid-js/web";

import "./index.css";
import Login from "./Login.tsx";
import Chat from "./Chat.tsx";
import { BackendProvider } from "./BackendContext";

const root = document.getElementById("root");

render(
  () => (
    <BackendProvider publicChildren={<Login />}>
      <Chat />
    </BackendProvider>
  ),
  root!,
);
