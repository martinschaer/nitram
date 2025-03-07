import { useContext } from "solid-js";

// -----------------------------------------------------------------------------
// Nitram bindings
//
import { GetTokenAPI } from "bindings/API";

// -----------------------------------------------------------------------------
// Local imports
//
import { BackendContext } from "./BackendContext";

// =============================================================================
// Component
// =============================================================================
function Login() {
  // -- HTML Elements
  let input!: HTMLInputElement;

  // -- Nitram context
  const { server } = useContext(BackendContext) ?? { server: null };
  if (!server) {
    throw new Error("BackendContext not found");
  }

  // -- Callbacks
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

  // -- Render
  return (
    <>
      <h1>Hello</h1>
      <input type="text" ref={(el) => (input = el)} placeholder="Name" />
      <button onClick={handleLogin}>Login</button>
    </>
  );
}

export default Login;
