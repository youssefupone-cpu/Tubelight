import { ping } from "@yoube/contracts";
import { useEffect, useState } from "react";

export function App() {
	const [pong, setPong] = useState<string>("…");
	useEffect(() => {
		ping()
			.then((r) => setPong(r.value))
			.catch(console.error);
	}, []);
	return (
		<main style={{ font: "16px system-ui", padding: 24 }}>
			Tubelight — {pong}
		</main>
	);
}
