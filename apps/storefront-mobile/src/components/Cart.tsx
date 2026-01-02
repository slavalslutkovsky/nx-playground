import { Component, For, Show } from 'solid-js';
import { A } from '@solidjs/router';
import { formatPrice } from '../lib/api';
import { cartItems, cartTotal, updateQuantity, removeFromCart, clearCart } from '../lib/cart-store';

const Cart: Component = () => {
  const items = () => Object.values(cartItems());

  return (
    <div class="cart">
      <h1 class="title">Shopping Cart</h1>

      <Show when={items().length === 0}>
        <div class="empty">
          <p>Your cart is empty</p>
          <A href="/" class="continue-link">Continue Shopping</A>
        </div>
      </Show>

      <Show when={items().length > 0}>
        <div class="items">
          <For each={items()}>
            {(item) => (
              <div class="cart-item">
                <div class="item-image">
                  {item.product.images[0] ? (
                    <img src={item.product.images[0].url} alt={item.product.name} />
                  ) : (
                    <div class="placeholder">No Image</div>
                  )}
                </div>
                <div class="item-info">
                  <h3 class="item-name">{item.product.name}</h3>
                  <p class="item-price">{formatPrice(item.product.price)}</p>
                  <div class="quantity-controls">
                    <button
                      onClick={() => updateQuantity(item.product.id, item.quantity - 1)}
                    >
                      -
                    </button>
                    <span>{item.quantity}</span>
                    <button
                      onClick={() => updateQuantity(item.product.id, item.quantity + 1)}
                    >
                      +
                    </button>
                  </div>
                </div>
                <div class="item-actions">
                  <span class="item-total">
                    {formatPrice(item.product.price * item.quantity)}
                  </span>
                  <button
                    class="remove-btn"
                    onClick={() => removeFromCart(item.product.id)}
                  >
                    Remove
                  </button>
                </div>
              </div>
            )}
          </For>
        </div>

        <div class="summary">
          <div class="total">
            <span>Total</span>
            <span class="total-price">{formatPrice(cartTotal())}</span>
          </div>
          <button class="checkout-btn">Proceed to Checkout</button>
          <button class="clear-btn" onClick={clearCart}>Clear Cart</button>
        </div>
      </Show>

      <style>{`
        .cart {
          display: flex;
          flex-direction: column;
          gap: 1rem;
        }
        .title {
          font-size: 1.5rem;
          margin: 0;
        }
        .empty {
          text-align: center;
          padding: 2rem;
        }
        .continue-link {
          color: #3b82f6;
          text-decoration: none;
        }
        .items {
          display: flex;
          flex-direction: column;
          gap: 1rem;
        }
        .cart-item {
          display: flex;
          gap: 1rem;
          background: white;
          padding: 1rem;
          border-radius: 0.5rem;
        }
        .item-image {
          width: 80px;
          height: 80px;
          flex-shrink: 0;
          background: #f3f4f6;
          border-radius: 0.5rem;
          overflow: hidden;
        }
        .item-image img {
          width: 100%;
          height: 100%;
          object-fit: cover;
        }
        .placeholder {
          display: flex;
          align-items: center;
          justify-content: center;
          height: 100%;
          color: #9ca3af;
          font-size: 0.75rem;
        }
        .item-info {
          flex: 1;
          min-width: 0;
        }
        .item-name {
          font-size: 0.875rem;
          font-weight: 600;
          margin: 0 0 0.25rem;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }
        .item-price {
          color: #3b82f6;
          margin: 0 0 0.5rem;
          font-size: 0.875rem;
        }
        .quantity-controls {
          display: flex;
          align-items: center;
          gap: 0.5rem;
        }
        .quantity-controls button {
          width: 28px;
          height: 28px;
          border: 1px solid #d1d5db;
          background: white;
          border-radius: 0.25rem;
          cursor: pointer;
        }
        .item-actions {
          display: flex;
          flex-direction: column;
          align-items: flex-end;
          gap: 0.5rem;
        }
        .item-total {
          font-weight: 600;
        }
        .remove-btn {
          color: #dc2626;
          background: none;
          border: none;
          font-size: 0.75rem;
          cursor: pointer;
        }
        .summary {
          background: white;
          padding: 1rem;
          border-radius: 0.5rem;
          display: flex;
          flex-direction: column;
          gap: 1rem;
        }
        .total {
          display: flex;
          justify-content: space-between;
          font-size: 1.125rem;
        }
        .total-price {
          font-weight: bold;
          color: #3b82f6;
        }
        .checkout-btn {
          width: 100%;
          padding: 1rem;
          background: #3b82f6;
          color: white;
          border: none;
          border-radius: 0.5rem;
          font-size: 1rem;
          font-weight: 600;
          cursor: pointer;
        }
        .clear-btn {
          width: 100%;
          padding: 0.75rem;
          background: white;
          color: #6b7280;
          border: 1px solid #d1d5db;
          border-radius: 0.5rem;
          cursor: pointer;
        }
      `}</style>
    </div>
  );
};

export default Cart;
