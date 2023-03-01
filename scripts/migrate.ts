import { readLines } from "https://deno.land/std@0.171.0/io/read_lines.ts";

async function startProcess() {
  const proxyProcess = Deno.run({
    cmd: ["flyctl", "proxy", "5432", "-a", Deno.env.get("POSTGRES_APP_NAME")],
    stdout: "piped",
  });

  for await (const line of readLines(proxyProcess.stdout)) {
    console.log(line);
    if (line.startsWith("Proxying local port 5432")) {
      return () => {
        proxyProcess.kill("SIGTERM");
        proxyProcess.close();
      };
    } else {
      throw new Error(`Failed to start fly proxy.`);
    }
  }
  throw new Error(`Failed to start fly proxy.`);
}

const endProcess = await startProcess();

const databaseUrl = Deno.env.get("DATABASE_URL");
const migrate = Deno.run({
  cmd: ["sqlx", "migrate", "run", "--database-url", databaseUrl],
});
await migrate.status();
migrate.close();
endProcess!();
