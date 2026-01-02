import { Component, createSignal, onMount, Show } from 'solid-js';
import { useParams, A } from '@solidjs/router';
import { fetchProduct, formatPrice, type Product } from '../lib/api';
import { addToCart } from '../lib/cart-store';

const ProductDetail: Component = () => {
  const params = useParams();
  const [product, setProduct] = createSignal<Product | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);

  onMount(async () => {
    try {
      const data = await fetchProduct(params.id);
      setProduct(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load product');
    } finally {
      setLoading(false);
    }
  });

  const handleAddToCart = () => {
    const p = product();
    if (p) addToCart(p);
  };

  return (
    <div class="product-detail">
      <A href="/" class="back-link">← Back</A>

      <Show when={loading()}>
        <div class="loading">Loading...</div>
      </Show>

      <Show when={error()}>
        <div class="error">{error()}</div>
      </Show>

      <Show when={!loading() && product()}>
        {(p) => {
          const primaryImage = p().images.find((img) => img.is_primary) || p().images[0];
          const isOutOfStock = p().stock <= p().reserved_stock;

          return (
            <>
              <div class="product-image">
                {primaryImage ? (
                  <img src={primaryImage.url} alt={primaryImage.alt || p().name} />
                ) : (
                  <div class="placeholder">No Image</div>
                )}
              </div>

              <div class="product-info">
                <h1 class="product-name">{p().name}</h1>
                {p().brand && <p class="product-brand">by {p().brand}</p>}

                <div class="price-section">
                  <span class="price">{formatPrice(p().price)}</span>
                  <span class={`stock ${isOutOfStock ? 'out' : 'in'}`}>
                    {isOutOfStock ? 'Out of Stock' : `${p().stock - p().reserved_stock} in stock`}
                  </span>
                </div>

                <p class="description">{p().description}</p>

                <button
                  class="add-to-cart-btn"
                  onClick={handleAddToCart}
                  disabled={isOutOfStock}
                >
                  Add to Cart
                </button>

                <div class="details">
                  <h3>Details</h3>
                  <dl>
                    <dt>Category</dt>
                    <dd>{p().category.replace('_', ' ')}</dd>
                    {p().sku && (
                      <>
                        <dt>SKU</dt>
                        <dd>{p().sku}</dd>
                      </>
                    )}
                    {p().weight && (
                      <>
                        <dt>Weight</dt>
                        <dd>{p().weight}g</dd>
                      </>
                    )}
                  </dl>
                </div>
              </div>
            </>
          );
        }}
      </Show>

      <style>{`
        .product-detail {
          display: flex;
          flex-direction: column;
          gap: 1rem;
        }
        .back-link {
          color: #3b82f6;
          text-decoration: none;
          font-size: 0.875rem;
        }
        .loading, .error {
          text-align: center;
          padding: 2rem;
        }
        .error {
          color: #dc2626;
        }
        .product-image {
          aspect-ratio: 1;
          background: #f3f4f6;
          border-radius: 0.75rem;
          overflow: hidden;
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
        .product-info {
          display: flex;
          flex-direction: column;
          gap: 1rem;
        }
        .product-name {
          font-size: 1.5rem;
          font-weight: bold;
          margin: 0;
        }
        .product-brand {
          color: #6b7280;
          margin: 0;
        }
        .price-section {
          display: flex;
          align-items: center;
          gap: 1rem;
        }
        .price {
          font-size: 1.5rem;
          font-weight: bold;
          color: #3b82f6;
        }
        .stock {
          font-size: 0.875rem;
          padding: 0.25rem 0.75rem;
          border-radius: 999px;
        }
        .stock.in {
          background: #d1fae5;
          color: #047857;
        }
        .stock.out {
          background: #fee2e2;
          color: #b91c1c;
        }
        .description {
          color: #4b5563;
          line-height: 1.6;
        }
        .add-to-cart-btn {
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
        .add-to-cart-btn:disabled {
          background: #9ca3af;
          cursor: not-allowed;
        }
        .details {
          background: white;
          padding: 1rem;
          border-radius: 0.5rem;
        }
        .details h3 {
          margin: 0 0 0.75rem;
          font-size: 1rem;
        }
        .details dl {
          display: grid;
          grid-template-columns: auto 1fr;
          gap: 0.5rem 1rem;
          margin: 0;
        }
        .details dt {
          color: #6b7280;
        }
        .details dd {
          margin: 0;
          text-transform: capitalize;
        }
      `}</style>
    </div>
  );
};

export default ProductDetail;
