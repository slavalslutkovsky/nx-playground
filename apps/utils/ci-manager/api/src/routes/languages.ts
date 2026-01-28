import { Hono } from 'hono';
import { languages } from '../data/languages';

const app = new Hono();

app.get('/', (c) => {
  return c.json(languages);
});

app.get('/:id', (c) => {
  const id = c.req.param('id');
  const language = languages.find((l) => l.id === id);

  if (!language) {
    return c.json({ error: 'Language not found' }, 404);
  }

  return c.json(language);
});

export default app;
