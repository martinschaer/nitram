import type { GetTokenAPI } from "bindings/API";

// Local imports
import { useBackend } from "./BackendContext";

// =============================================================================
// Component
// =============================================================================
function Login() {
  // -- HTML Elements
  let input!: HTMLInputElement;

  // -- Nitram context
  const { server } = useBackend();

  // -- Callbacks
  const handleLogin = () => {
    server()
      .request<GetTokenAPI>({
        method: "GetToken",
        params: { user_name: input.value },
      })
      .then((token) => {
        server().auth(token);
      });
  };

  // -- Render
  return (
    <>
      <h1>Hello</h1>
      <input type="text" ref={(el) => { input = el }} placeholder="Name" />
      <button type="button" onClick={handleLogin}>Login</button>
    </>
  );
}

export default Login;
