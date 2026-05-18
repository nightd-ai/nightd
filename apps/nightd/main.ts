import { Hono } from "hono";

const app = new Hono();

app.get("/", (c) => {
  return c.text("Hello nightd!");
});

Deno.serve(app.fetch);
