import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export function App() {
  const [pong, setPong] = useState<string>("…");
  useEffect(() => { invoke<string>("ping").then(setPong).catch(console.error); }, []);
  return <main style={{ font: "16px system-ui", padding: 24 }}>yoube — {pong}</main>;
}
