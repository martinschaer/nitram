import { useContext } from "solid-js";

import { BackendContext } from "./BackendContext";
import { SendMessageAPI } from "nitram/API";

function Private() {
  let input!: HTMLInputElement;
  const { server } = useContext(BackendContext) ?? { server: null };
  if (!server) {
    throw new Error("BackendContext not found");
  }

  const handleLogout = () => {
    server().logout();
  };

  const handleMethod = () => {
    input.disabled = true;
    server()
      .request<SendMessageAPI>({
        id: "fake",
        method: "SendMessage",
        params: { message: input.value },
      })
      .then(() => {
        input.value = "";
      })
      .finally(() => {
        input.disabled = false;
      });
  };

  return (
    <>
      <h1>Private</h1>
      <button onClick={handleLogout}>Logout</button>
      <div>
        <h2>Chat</h2>
        <input type="text" ref={(el) => (input = el)} placeholder="Message" />
        <button onClick={handleMethod}>Send</button>
      </div>
    </>
  );
}

export default Private;
