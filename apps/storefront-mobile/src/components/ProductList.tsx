import { Component, createSignal, onMount, For, Show } from 'solid-js';
import { A } from '@solidjs/router';
import { fetchProducts, formatPrice, type Product } from '../lib/api';
import { addToCart } from '../lib/cart-store';

const ProductList: Component = () => {
  const [products, setProducts] = createSignal<Product[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [search, setSearch] = createSignal('');

  onMount(async () => {
    try {
      const data = await fetchProducts({ status: 'active', limit: 50 });
      setProducts(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load products');
    } finally {
      setLoading(false);
    }
  });

  const filteredProducts = () => {
    const searchTerm = search().toLowerCase();
    if (!searchTerm) return products();
    return products().filter(
      (p) =>
        p.name.toLowerCase().includes(searchTerm) ||
        p.description.toLowerCase().includes(searchTerm)
    );
  };

  const handleAddToCart = (product: Product, e: Event) => {
    e.preventDefault();
    e.stopPropagation();
    addToCart(product);
  };

  return (
    <div class="product-list">
      <div class="search-bar">
        <input
          type="text"
          placeholder="Search products..."
          value={search()}
          onInput={(e) => setSearch(e.currentTarget.value)}
          class="search-input"
        />
      </div>

      <Show when={loading()}>
        <div class="loading">Loading products...</div>
      </Show>

      <Show when={error()}>
        <div class="error">{error()}</div>
      </Show>

      <Show when={!loading() && !error()}>
        <div class="grid">
          <For each={filteredProducts()}>
            {(product) => {
              const primaryImage = product.images.find((img) => img.is_primary) || product.images[0];
              const isOutOfStock = product.stock <= product.reserved_stock;

              return (
                <A href={`/products/${product.id}`} class="product-card">
                  <div class="product-image">
                    {primaryImage ? (
                      <img src={primaryImage.url} alt={primaryImage.alt || product.name} />
                    ) : (
                      <div class="placeholder">No Image</div>
                    )}
                    {isOutOfStock && <span class="out-of-stock">Out of Stock</span>}
                  </div>
                  <div class="product-info">
                    <h3 class="product-name">{product.name}</h3>
                    {product.brand && <p class="product-brand">{product.brand}</p>}
                    <div class="product-footer">
                      <span class="product-price">{formatPrice(product.price)}</span>
                      <button
                        class="add-btn"
                        onClick={(e) => handleAddToCart(product, e)}
                        disabled={isOutOfStock}
                      >
                        Add
                      </button>
                    </div>
                  </div>
                </A>
              );
            }}
          </For>
        </div>
      </Show>

      <style>{`
        .product-list {
          display: flex;
          flex-direction: column;
          gap: 1rem;
        }
        .search-bar {
          position: sticky;
          top: 0;
          background: #f9fafb;
          padding: 0.5rem 0;
          z-index: 10;
        }
        .search-input {
          width: 100%;
          padding: 0.75rem 1rem;
          border: 1px solid #d1d5db;
          border-radius: 0.5rem;
          font-size: 1rem;
        }
        .loading, .error {
          text-align: center;
          padding: 2rem;
        }
        .error {
          color: #dc2626;
        }
        .grid {
          display: grid;
          grid-template-columns: repeat(2, 1fr);
          gap: 1rem;
        }
        .product-card {
          background: white;
          border-radius: 0.75rem;
          overflow: hidden;
          box-shadow: 0 1px 3px rgba(0,0,0,0.1);
          text-decoration: none;
          color: inherit;
        }
        .product-image {
          position: relative;
          aspect-ratio: 1;
          background: #f3f4f6;
        }
        .product-image img {
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
        }
        .out-of-stock {
          position: absolute;
          top: 0.5rem;
          right: 0.5rem;
          background: #ef4444;
          color: white;
          font-size: 0.625rem;
          padding: 0.125rem 0.5rem;
          border-radius: 0.25rem;
        }
        .product-info {
          padding: 0.75rem;
        }
        .product-name {
          font-size: 0.875rem;
          font-weight: 600;
          margin: 0 0 0.25rem;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }
        .product-brand {
          font-size: 0.75rem;
          color: #6b7280;
          margin: 0 0 0.5rem;
        }
        .product-footer {
          display: flex;
          align-items: center;
          justify-content: space-between;
        }
        .product-price {
          font-weight: bold;
          color: #3b82f6;
        }
        .add-btn {
          padding: 0.375rem 0.75rem;
          background: #3b82f6;
          color: white;
          border: none;
          border-radius: 0.375rem;
          font-size: 0.75rem;
          cursor: pointer;
        }
        .add-btn:disabled {
          background: #9ca3af;
          cursor: not-allowed;
        }
      `}</style>
    </div>
  );
};

export default ProductList;
