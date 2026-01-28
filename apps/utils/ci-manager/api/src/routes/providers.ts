import { Hono } from 'hono';
import { providers } from '../data/providers';

const app = new Hono();

app.get('/', (c) => {
  return c.json(providers);
});

app.get('/:id', (c) => {
  const id = c.req.param('id');
  const provider = providers.find((p) => p.id === id);

  if (!provider) {
    return c.json({ error: 'Provider not found' }, 404);
  }

  return c.json(provider);
});

export default app;
