import { Injectable, NotFoundException, ConflictException, BadRequestException, Inject } from '@nestjs/common';
import { ClientProxy } from '@nestjs/microservices';
import { v4 as uuidv4 } from 'uuid';
import {
  CreateProductDto,
  UpdateProductDto,
  ProductDto,
  ProductFilterDto,
  ProductStatus,
  StockAdjustmentDto,
  ReserveStockDto,
} from './dto/product.dto';

// In-memory store for demo (replace with MongoDB in production)
const products: Map<string, ProductDto> = new Map();

@Injectable()
export class ProductsService {
  constructor(
    @Inject('NATS_SERVICE') private readonly natsClient: ClientProxy,
  ) {}

  async create(dto: CreateProductDto): Promise<ProductDto> {
    // Check for duplicate SKU
    if (dto.sku) {
      const existing = Array.from(products.values()).find(p => p.sku === dto.sku);
      if (existing) {
        throw new ConflictException(`Product with SKU '${dto.sku}' already exists`);
      }
    }

    const now = new Date().toISOString();
    const product: ProductDto = {
      id: uuidv4(),
      name: dto.name,
      description: dto.description,
      price: dto.price,
      displayPrice: dto.price / 100,
      stock: dto.stock,
      reservedStock: 0,
      category: dto.category,
      status: ProductStatus.Active,
      images: dto.images || [],
      sku: dto.sku,
      barcode: dto.barcode,
      brand: dto.brand,
      weight: dto.weight,
      tags: dto.tags || [],
      createdAt: now,
      updatedAt: now,
    };

    products.set(product.id, product);

    // Publish event to NATS
    this.natsClient.emit('products.created', product);

    return product;
  }

  async findById(id: string): Promise<ProductDto> {
    const product = products.get(id);
    if (!product) {
      throw new NotFoundException(`Product not found: ${id}`);
    }
    return product;
  }

  async findBySku(sku: string): Promise<ProductDto> {
    const product = Array.from(products.values()).find(p => p.sku === sku);
    if (!product) {
      throw new NotFoundException(`Product with SKU '${sku}' not found`);
    }
    return product;
  }

  async findAll(filter: ProductFilterDto): Promise<ProductDto[]> {
    let result = Array.from(products.values());

    if (filter.status) {
      result = result.filter(p => p.status === filter.status);
    }
    if (filter.category) {
      result = result.filter(p => p.category === filter.category);
    }
    if (filter.brand) {
      result = result.filter(p => p.brand === filter.brand);
    }
    if (filter.minPrice !== undefined) {
      result = result.filter(p => p.price >= filter.minPrice);
    }
    if (filter.maxPrice !== undefined) {
      result = result.filter(p => p.price <= filter.maxPrice);
    }
    if (filter.inStock !== undefined) {
      result = result.filter(p => filter.inStock ? (p.stock - p.reservedStock) > 0 : true);
    }
    if (filter.search) {
      const search = filter.search.toLowerCase();
      result = result.filter(p =>
        p.name.toLowerCase().includes(search) ||
        p.description.toLowerCase().includes(search)
      );
    }

    const offset = filter.offset || 0;
    const limit = filter.limit || 20;

    return result.slice(offset, offset + limit);
  }

  async update(id: string, dto: UpdateProductDto): Promise<ProductDto> {
    const product = await this.findById(id);

    // Check for duplicate SKU
    if (dto.sku && dto.sku !== product.sku) {
      const existing = Array.from(products.values()).find(p => p.sku === dto.sku);
      if (existing) {
        throw new ConflictException(`Product with SKU '${dto.sku}' already exists`);
      }
    }

    const updated: ProductDto = {
      ...product,
      ...dto,
      displayPrice: dto.price !== undefined ? dto.price / 100 : product.displayPrice,
      updatedAt: new Date().toISOString(),
    };

    products.set(id, updated);

    // Publish event to NATS
    this.natsClient.emit('products.updated', updated);

    return updated;
  }

  async delete(id: string): Promise<void> {
    const product = await this.findById(id);
    products.delete(id);

    // Publish event to NATS
    this.natsClient.emit('products.deleted', { id, product });
  }

  async adjustStock(id: string, dto: StockAdjustmentDto): Promise<ProductDto> {
    const product = await this.findById(id);
    const newStock = product.stock + dto.quantity;

    if (newStock < 0) {
      throw new BadRequestException('Stock cannot be negative');
    }

    const updated: ProductDto = {
      ...product,
      stock: newStock,
      updatedAt: new Date().toISOString(),
    };

    products.set(id, updated);

    // Publish event to NATS
    this.natsClient.emit('products.stock.adjusted', {
      productId: id,
      quantity: dto.quantity,
      reason: dto.reason,
      newStock,
    });

    return updated;
  }

  async reserveStock(id: string, dto: ReserveStockDto): Promise<{ reservationId: string; product: ProductDto }> {
    const product = await this.findById(id);
    const availableStock = product.stock - product.reservedStock;

    if (dto.quantity > availableStock) {
      throw new BadRequestException(`Insufficient stock: available ${availableStock}, requested ${dto.quantity}`);
    }

    const updated: ProductDto = {
      ...product,
      reservedStock: product.reservedStock + dto.quantity,
      updatedAt: new Date().toISOString(),
    };

    products.set(id, updated);

    const reservationId = uuidv4();

    // Publish event to NATS
    this.natsClient.emit('products.stock.reserved', {
      reservationId,
      productId: id,
      quantity: dto.quantity,
      orderId: dto.orderId,
    });

    return { reservationId, product: updated };
  }

  async releaseStock(id: string, quantity: number): Promise<ProductDto> {
    const product = await this.findById(id);

    if (quantity > product.reservedStock) {
      throw new BadRequestException(`Cannot release more than reserved: reserved ${product.reservedStock}, releasing ${quantity}`);
    }

    const updated: ProductDto = {
      ...product,
      reservedStock: product.reservedStock - quantity,
      updatedAt: new Date().toISOString(),
    };

    products.set(id, updated);

    // Publish event to NATS
    this.natsClient.emit('products.stock.released', {
      productId: id,
      quantity,
    });

    return updated;
  }

  async commitStock(id: string, quantity: number): Promise<ProductDto> {
    const product = await this.findById(id);

    if (quantity > product.reservedStock) {
      throw new BadRequestException(`Cannot commit more than reserved: reserved ${product.reservedStock}, committing ${quantity}`);
    }

    const updated: ProductDto = {
      ...product,
      stock: product.stock - quantity,
      reservedStock: product.reservedStock - quantity,
      updatedAt: new Date().toISOString(),
    };

    products.set(id, updated);

    // Publish event to NATS
    this.natsClient.emit('products.stock.committed', {
      productId: id,
      quantity,
    });

    return updated;
  }

  async getLowStock(threshold: number = 10): Promise<ProductDto[]> {
    return Array.from(products.values()).filter(p => p.stock <= threshold);
  }

  async count(): Promise<number> {
    return products.size;
  }
}
