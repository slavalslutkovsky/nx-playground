/**
 * Cart state management with nanostores
 */
import { atom, computed, map } from 'nanostores';
import type { Product } from './api-client';

export interface CartItem {
  product: Product;
  quantity: number;
}

// Cart items stored by product ID
export const cartItems = map<Record<string, CartItem>>({});

// Cart visibility
export const isCartOpen = atom(false);

// Computed values
export const cartCount = computed(cartItems, (items) => {
  return Object.values(items).reduce((total, item) => total + item.quantity, 0);
});

export const cartTotal = computed(cartItems, (items) => {
  return Object.values(items).reduce(
    (total, item) => total + item.product.price * item.quantity,
    0
  );
});

// Actions
export function addToCart(product: Product, quantity = 1) {
  const existingItem = cartItems.get()[product.id];

  if (existingItem) {
    cartItems.setKey(product.id, {
      ...existingItem,
      quantity: existingItem.quantity + quantity,
    });
  } else {
    cartItems.setKey(product.id, { product, quantity });
  }
}

export function removeFromCart(productId: string) {
  const items = { ...cartItems.get() };
  delete items[productId];
  cartItems.set(items);
}

export function updateQuantity(productId: string, quantity: number) {
  if (quantity <= 0) {
    removeFromCart(productId);
    return;
  }

  const existingItem = cartItems.get()[productId];
  if (existingItem) {
    cartItems.setKey(productId, {
      ...existingItem,
      quantity,
    });
  }
}

export function clearCart() {
  cartItems.set({});
}

export function toggleCart() {
  isCartOpen.set(!isCartOpen.get());
}

export function openCart() {
  isCartOpen.set(true);
}

export function closeCart() {
  isCartOpen.set(false);
}
