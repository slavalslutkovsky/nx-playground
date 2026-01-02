import {
  Controller,
  Get,
  Post,
  Put,
  Delete,
  Body,
  Param,
  Query,
  HttpCode,
  HttpStatus,
} from '@nestjs/common';
import {
  ApiTags,
  ApiOperation,
  ApiResponse,
  ApiParam,
  ApiQuery,
} from '@nestjs/swagger';
import { ProductsService } from './products.service';
import {
  CreateProductDto,
  UpdateProductDto,
  ProductDto,
  ProductFilterDto,
  StockAdjustmentDto,
  ReserveStockDto,
} from './dto/product.dto';

@ApiTags('products')
@Controller('api/products')
export class ProductsController {
  constructor(private readonly productsService: ProductsService) {}

  @Post()
  @ApiOperation({ summary: 'Create a new product' })
  @ApiResponse({ status: 201, description: 'Product created', type: ProductDto })
  @ApiResponse({ status: 409, description: 'Duplicate SKU' })
  async create(@Body() dto: CreateProductDto): Promise<ProductDto> {
    return this.productsService.create(dto);
  }

  @Get()
  @ApiOperation({ summary: 'List products with optional filters' })
  @ApiResponse({ status: 200, description: 'List of products', type: [ProductDto] })
  async findAll(@Query() filter: ProductFilterDto): Promise<ProductDto[]> {
    return this.productsService.findAll(filter);
  }

  @Get('count')
  @ApiOperation({ summary: 'Get total product count' })
  @ApiResponse({ status: 200, description: 'Product count' })
  async count(): Promise<{ count: number }> {
    const count = await this.productsService.count();
    return { count };
  }

  @Get('low-stock')
  @ApiOperation({ summary: 'Get products with low stock' })
  @ApiQuery({ name: 'threshold', required: false, type: Number })
  @ApiResponse({ status: 200, description: 'Low stock products', type: [ProductDto] })
  async getLowStock(@Query('threshold') threshold?: number): Promise<ProductDto[]> {
    return this.productsService.getLowStock(threshold);
  }

  @Get('sku/:sku')
  @ApiOperation({ summary: 'Get product by SKU' })
  @ApiParam({ name: 'sku', description: 'Product SKU' })
  @ApiResponse({ status: 200, description: 'Product found', type: ProductDto })
  @ApiResponse({ status: 404, description: 'Product not found' })
  async findBySku(@Param('sku') sku: string): Promise<ProductDto> {
    return this.productsService.findBySku(sku);
  }

  @Get(':id')
  @ApiOperation({ summary: 'Get product by ID' })
  @ApiParam({ name: 'id', description: 'Product UUID' })
  @ApiResponse({ status: 200, description: 'Product found', type: ProductDto })
  @ApiResponse({ status: 404, description: 'Product not found' })
  async findById(@Param('id') id: string): Promise<ProductDto> {
    return this.productsService.findById(id);
  }

  @Put(':id')
  @ApiOperation({ summary: 'Update a product' })
  @ApiParam({ name: 'id', description: 'Product UUID' })
  @ApiResponse({ status: 200, description: 'Product updated', type: ProductDto })
  @ApiResponse({ status: 404, description: 'Product not found' })
  async update(
    @Param('id') id: string,
    @Body() dto: UpdateProductDto,
  ): Promise<ProductDto> {
    return this.productsService.update(id, dto);
  }

  @Delete(':id')
  @HttpCode(HttpStatus.NO_CONTENT)
  @ApiOperation({ summary: 'Delete a product' })
  @ApiParam({ name: 'id', description: 'Product UUID' })
  @ApiResponse({ status: 204, description: 'Product deleted' })
  @ApiResponse({ status: 404, description: 'Product not found' })
  async delete(@Param('id') id: string): Promise<void> {
    return this.productsService.delete(id);
  }

  @Post(':id/stock')
  @ApiOperation({ summary: 'Adjust product stock' })
  @ApiParam({ name: 'id', description: 'Product UUID' })
  @ApiResponse({ status: 200, description: 'Stock adjusted', type: ProductDto })
  async adjustStock(
    @Param('id') id: string,
    @Body() dto: StockAdjustmentDto,
  ): Promise<ProductDto> {
    return this.productsService.adjustStock(id, dto);
  }

  @Post(':id/reserve')
  @ApiOperation({ summary: 'Reserve product stock' })
  @ApiParam({ name: 'id', description: 'Product UUID' })
  @ApiResponse({ status: 200, description: 'Stock reserved' })
  async reserveStock(
    @Param('id') id: string,
    @Body() dto: ReserveStockDto,
  ): Promise<{ reservationId: string; product: ProductDto }> {
    return this.productsService.reserveStock(id, dto);
  }

  @Post(':id/release')
  @ApiOperation({ summary: 'Release reserved stock' })
  @ApiParam({ name: 'id', description: 'Product UUID' })
  @ApiResponse({ status: 200, description: 'Stock released', type: ProductDto })
  async releaseStock(
    @Param('id') id: string,
    @Body() dto: { quantity: number },
  ): Promise<ProductDto> {
    return this.productsService.releaseStock(id, dto.quantity);
  }

  @Post(':id/commit')
  @ApiOperation({ summary: 'Commit reserved stock (finalize sale)' })
  @ApiParam({ name: 'id', description: 'Product UUID' })
  @ApiResponse({ status: 200, description: 'Stock committed', type: ProductDto })
  async commitStock(
    @Param('id') id: string,
    @Body() dto: { quantity: number },
  ): Promise<ProductDto> {
    return this.productsService.commitStock(id, dto.quantity);
  }
}
