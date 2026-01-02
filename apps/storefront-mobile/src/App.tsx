import { Component, JSX } from 'solid-js';
import { A } from '@solidjs/router';
import { cartCount } from './lib/cart-store';

interface AppProps {
  children?: JSX.Element;
}

const App: Component<AppProps> = (props) => {
  return (
    <div class="app">
      <header class="header">
        <A href="/" class="logo">Storefront</A>
        <nav class="nav">
          <A href="/cart" class="cart-link">
            Cart
            {cartCount() > 0 && (
              <span class="cart-badge">{cartCount()}</span>
            )}
          </A>
        </nav>
      </header>
      <main class="main">
        {props.children}
      </main>
      <style>{`
        .app {
          min-height: 100%;
          display: flex;
          flex-direction: column;
        }
        .header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 1rem;
          background: white;
          border-bottom: 1px solid #e5e7eb;
          position: sticky;
          top: 0;
          z-index: 100;
        }
        .logo {
          font-size: 1.25rem;
          font-weight: bold;
          color: #3b82f6;
          text-decoration: none;
        }
        .nav {
          display: flex;
          align-items: center;
          gap: 1rem;
        }
        .cart-link {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          color: #374151;
          text-decoration: none;
        }
        .cart-badge {
          background: #3b82f6;
          color: white;
          font-size: 0.75rem;
          padding: 0.125rem 0.5rem;
          border-radius: 999px;
        }
        .main {
          flex: 1;
          padding: 1rem;
        }
      `}</style>
    </div>
  );
};

export default App;
