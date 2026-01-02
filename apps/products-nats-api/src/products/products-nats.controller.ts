import { Controller, Logger } from '@nestjs/common';
import { MessagePattern, EventPattern, Payload } from '@nestjs/microservices';
import { ProductsService } from './products.service';
import {
  CreateProductDto,
  UpdateProductDto,
  ProductDto,
  ProductFilterDto,
  StockAdjustmentDto,
  ReserveStockDto,
} from './dto/product.dto';

/**
 * NATS Message Handler Controller
 *
 * Handles both request-response (MessagePattern) and event-based (EventPattern) messages.
 *
 * Message Patterns (request-response):
 * - products.create
 * - products.get
 * - products.getbySku
 * - products.list
 * - products.update
 * - products.delete
 * - products.stock.adjust
 * - products.stock.reserve
 * - products.stock.release
 * - products.stock.commit
 * - products.low-stock
 *
 * Event Patterns (fire-and-forget):
 * - products.sync
 * - products.cache.invalidate
 */
@Controller()
export class ProductsNatsController {
  private readonly logger = new Logger(ProductsNatsController.name);

  constructor(private readonly productsService: ProductsService) {}

  // ============================================================
  // Request-Response Message Patterns
  // ============================================================

  @MessagePattern('products.create')
  async handleCreate(@Payload() dto: CreateProductDto): Promise<ProductDto> {
    this.logger.log(`NATS: Creating product "${dto.name}"`);
    return this.productsService.create(dto);
  }

  @MessagePattern('products.get')
  async handleGet(@Payload() data: { id: string }): Promise<ProductDto> {
    this.logger.log(`NATS: Getting product ${data.id}`);
    return this.productsService.findById(data.id);
  }

  @MessagePattern('products.getBySku')
  async handleGetBySku(@Payload() data: { sku: string }): Promise<ProductDto> {
    this.logger.log(`NATS: Getting product by SKU ${data.sku}`);
    return this.productsService.findBySku(data.sku);
  }

  @MessagePattern('products.list')
  async handleList(@Payload() filter: ProductFilterDto): Promise<ProductDto[]> {
    this.logger.log(`NATS: Listing products with filter`);
    return this.productsService.findAll(filter);
  }

  @MessagePattern('products.update')
  async handleUpdate(
    @Payload() data: { id: string; update: UpdateProductDto },
  ): Promise<ProductDto> {
    this.logger.log(`NATS: Updating product ${data.id}`);
    return this.productsService.update(data.id, data.update);
  }

  @MessagePattern('products.delete')
  async handleDelete(@Payload() data: { id: string }): Promise<{ success: boolean }> {
    this.logger.log(`NATS: Deleting product ${data.id}`);
    await this.productsService.delete(data.id);
    return { success: true };
  }

  @MessagePattern('products.stock.adjust')
  async handleStockAdjust(
    @Payload() data: { id: string; adjustment: StockAdjustmentDto },
  ): Promise<ProductDto> {
    this.logger.log(`NATS: Adjusting stock for product ${data.id}`);
    return this.productsService.adjustStock(data.id, data.adjustment);
  }

  @MessagePattern('products.stock.reserve')
  async handleStockReserve(
    @Payload() data: { id: string; reservation: ReserveStockDto },
  ): Promise<{ reservationId: string; product: ProductDto }> {
    this.logger.log(`NATS: Reserving stock for product ${data.id}`);
    return this.productsService.reserveStock(data.id, data.reservation);
  }

  @MessagePattern('products.stock.release')
  async handleStockRelease(
    @Payload() data: { id: string; quantity: number },
  ): Promise<ProductDto> {
    this.logger.log(`NATS: Releasing stock for product ${data.id}`);
    return this.productsService.releaseStock(data.id, data.quantity);
  }

  @MessagePattern('products.stock.commit')
  async handleStockCommit(
    @Payload() data: { id: string; quantity: number },
  ): Promise<ProductDto> {
    this.logger.log(`NATS: Committing stock for product ${data.id}`);
    return this.productsService.commitStock(data.id, data.quantity);
  }

  @MessagePattern('products.low-stock')
  async handleLowStock(
    @Payload() data: { threshold?: number },
  ): Promise<ProductDto[]> {
    this.logger.log(`NATS: Getting low stock products`);
    return this.productsService.getLowStock(data.threshold);
  }

  @MessagePattern('products.count')
  async handleCount(): Promise<{ count: number }> {
    this.logger.log(`NATS: Getting product count`);
    const count = await this.productsService.count();
    return { count };
  }

  // ============================================================
  // Event Patterns (fire-and-forget)
  // ============================================================

  @EventPattern('products.sync')
  async handleSync(@Payload() data: { source: string }): Promise<void> {
    this.logger.log(`NATS Event: Sync requested from ${data.source}`);
    // In a real implementation, this would trigger a sync with external systems
  }

  @EventPattern('products.cache.invalidate')
  async handleCacheInvalidate(
    @Payload() data: { productIds?: string[]; all?: boolean },
  ): Promise<void> {
    if (data.all) {
      this.logger.log('NATS Event: Invalidating all product cache');
    } else if (data.productIds) {
      this.logger.log(`NATS Event: Invalidating cache for products: ${data.productIds.join(', ')}`);
    }
    // In a real implementation, this would clear Redis cache
  }

  @EventPattern('orders.completed')
  async handleOrderCompleted(
    @Payload() data: { orderId: string; items: Array<{ productId: string; quantity: number }> },
  ): Promise<void> {
    this.logger.log(`NATS Event: Order ${data.orderId} completed, committing stock`);
    for (const item of data.items) {
      try {
        await this.productsService.commitStock(item.productId, item.quantity);
      } catch (error) {
        this.logger.error(`Failed to commit stock for product ${item.productId}: ${error}`);
      }
    }
  }

  @EventPattern('orders.cancelled')
  async handleOrderCancelled(
    @Payload() data: { orderId: string; items: Array<{ productId: string; quantity: number }> },
  ): Promise<void> {
    this.logger.log(`NATS Event: Order ${data.orderId} cancelled, releasing stock`);
    for (const item of data.items) {
      try {
        await this.productsService.releaseStock(item.productId, item.quantity);
      } catch (error) {
        this.logger.error(`Failed to release stock for product ${item.productId}: ${error}`);
      }
    }
  }
}
