import { useContext } from "solid-js";

import { BackendContext } from "./BackendContext";
import { GetTokenAPI } from "nitram/API";

function App() {
  let input!: HTMLInputElement;
  const { server } = useContext(BackendContext) ?? { server: null };
  if (!server) {
    throw new Error("BackendContext not found");
  }

  const handleLogin = () => {
    server()
      .request<GetTokenAPI>({
        id: "fake",
        method: "GetToken",
        params: { user_name: input.value },
      })
      .then((token) => {
        server().auth(token);
      });
  };

  return (
    <>
      <h1>Hello</h1>
      <input type="text" ref={(el) => (input = el)} placeholder="Name" />
      <button onClick={handleLogin}>Login</button>
    </>
  );
}

export default App;
