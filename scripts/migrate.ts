import { readLines } from "https://deno.land/std@0.178.0/io/mod.ts";

async function startProcess() {
  const postgresApp = Deno.env.get("POSTGRES_APP_NAME");
  if (!postgresApp) {
    throw new Error(`POSTGRES_APP_NAME is not set.`);
  }
  const proxyProcess = Deno.run({
    cmd: ["flyctl", "proxy", "5432", "-a", postgresApp],
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
if (!databaseUrl) {
  throw new Error(`DATABASE_URL is not set.`);
}
const migrate = Deno.run({
  cmd: ["sqlx", "migrate", "run", "--database-url", databaseUrl],
});
await migrate.status();
migrate.close();
endProcess!();
