import { useState, useEffect } from "react";
import { ensureWasmReady } from "@/wasm/init";

export function useWasm() {
  const [ready, setReady] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    ensureWasmReady()
      .then(() => setReady(true))
      .catch((err) => setError(err instanceof Error ? err : new Error(String(err))));
  }, []);

  return { ready, error };
}
