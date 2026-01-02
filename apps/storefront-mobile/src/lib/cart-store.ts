/**
 * Cart state management with Solid.js signals
 */
import { createSignal, createMemo } from 'solid-js';
import type { Product } from './api';

export interface CartItem {
  product: Product;
  quantity: number;
}

// Cart state using signals
const [items, setItems] = createSignal<Record<string, CartItem>>({});

// Computed values
export const cartItems = items;

export const cartCount = createMemo(() => {
  return Object.values(items()).reduce((total, item) => total + item.quantity, 0);
});

export const cartTotal = createMemo(() => {
  return Object.values(items()).reduce(
    (total, item) => total + item.product.price * item.quantity,
    0
  );
});

// Actions
export function addToCart(product: Product, quantity = 1) {
  const currentItems = items();
  const existingItem = currentItems[product.id];

  if (existingItem) {
    setItems({
      ...currentItems,
      [product.id]: {
        ...existingItem,
        quantity: existingItem.quantity + quantity,
      },
    });
  } else {
    setItems({
      ...currentItems,
      [product.id]: { product, quantity },
    });
  }
}

export function removeFromCart(productId: string) {
  const currentItems = { ...items() };
  delete currentItems[productId];
  setItems(currentItems);
}

export function updateQuantity(productId: string, quantity: number) {
  if (quantity <= 0) {
    removeFromCart(productId);
    return;
  }

  const currentItems = items();
  const existingItem = currentItems[productId];
  if (existingItem) {
    setItems({
      ...currentItems,
      [productId]: {
        ...existingItem,
        quantity,
      },
    });
  }
}

export function clearCart() {
  setItems({});
}
