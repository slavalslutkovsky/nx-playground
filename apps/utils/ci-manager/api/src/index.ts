import { Hono } from 'hono';
import { cors } from 'hono/cors';
import languagesRoutes from './routes/languages';
import pipelinesRoutes from './routes/pipelines';
import providersRoutes from './routes/providers';

const app = new Hono();

app.use(
  '*',
  cors({
    origin: ['http://localhost:3000', 'http://localhost:5173'],
    allowMethods: ['GET', 'POST', 'OPTIONS'],
    allowHeaders: ['Content-Type'],
  }),
);

app.get('/', (c) => {
  return c.json({
    name: 'CI Pipeline Manager API',
    version: '0.0.1',
    endpoints: ['/api/languages', '/api/providers', '/api/pipelines/generate'],
  });
});

app.route('/api/languages', languagesRoutes);
app.route('/api/providers', providersRoutes);
app.route('/api/pipelines', pipelinesRoutes);

const port = Number(process.env.PORT) || 3001;

const server = Bun.serve({
  port,
  fetch: app.fetch,
});

console.log(`CI Manager API running on http://localhost:${server.port}`);
