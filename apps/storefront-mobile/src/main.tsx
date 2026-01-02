import { render } from 'solid-js/web';
import { Router, Route } from '@solidjs/router';
import App from './App';
import ProductList from './components/ProductList';
import ProductDetail from './components/ProductDetail';
import Cart from './components/Cart';

render(
  () => (
    <Router root={App}>
      <Route path="/" component={ProductList} />
      <Route path="/products/:id" component={ProductDetail} />
      <Route path="/cart" component={Cart} />
    </Router>
  ),
  document.getElementById('root')!
);
