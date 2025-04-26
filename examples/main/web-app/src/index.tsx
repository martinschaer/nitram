/* @refresh reload */
import { render } from "solid-js/web";

import "./index.css";
import Login from "./Login.tsx";
import Chat from "./Chat.tsx";
import { BackendProvider } from "./BackendContext";
import User from "./User.tsx";

const root = document.getElementById("root");

render(
  () => (
    <BackendProvider publicChildren={<Login />}>
      <User />
      <Chat />
    </BackendProvider>
  ),
  root!,
);
