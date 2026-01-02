import { NestFactory } from '@nestjs/core';
import { MicroserviceOptions, Transport } from '@nestjs/microservices';
import { ValidationPipe } from '@nestjs/common';
import { DocumentBuilder, SwaggerModule } from '@nestjs/swagger';
import { AppModule } from './app.module';

async function bootstrap() {
  // Create HTTP application
  const app = await NestFactory.create(AppModule);

  // Enable CORS
  app.enableCors();

  // Global validation pipe
  app.useGlobalPipes(
    new ValidationPipe({
      whitelist: true,
      forbidNonWhitelisted: true,
      transform: true,
      transformOptions: {
        enableImplicitConversion: true,
      },
    }),
  );

  // Swagger/OpenAPI documentation
  const config = new DocumentBuilder()
    .setTitle('Products NATS API')
    .setDescription('Products microservice with NATS messaging')
    .setVersion('1.0')
    .addTag('products')
    .addTag('health')
    .build();

  const document = SwaggerModule.createDocument(app, config);
  SwaggerModule.setup('docs', app, document);

  // Connect NATS microservice
  const natsUrl = process.env.NATS_URL || 'nats://localhost:4222';

  app.connectMicroservice<MicroserviceOptions>({
    transport: Transport.NATS,
    options: {
      servers: [natsUrl],
      queue: 'products-queue',
    },
  });

  // Start all microservices
  await app.startAllMicroservices();

  // Start HTTP server
  const port = process.env.APP_PORT || 3010;
  await app.listen(port);

  console.log(`HTTP server running on http://localhost:${port}`);
  console.log(`Swagger docs at http://localhost:${port}/docs`);
  console.log(`NATS microservice connected to ${natsUrl}`);
}

bootstrap();
