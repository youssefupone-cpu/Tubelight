import { useEffect, useState } from "react";
import { ping } from "@yoube/contracts";

export function App() {
  const [pong, setPong] = useState<string>("…");
  useEffect(() => { ping().then(r => setPong(r.value)).catch(console.error); }, []);
  return <main style={{ font: "16px system-ui", padding: 24 }}>yoube — {pong}</main>;
}
