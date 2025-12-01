# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a blog backend API written in Rust using a hexagonal/clean architecture pattern with four distinct layers organized as a Cargo workspace.

## Architecture

The codebase follows hexagonal architecture (ports and adapters) with clear separation of concerns:

### Layer Structure

1. **blog-domain** - Core domain layer
   - Contains domain models and business entities (articles, users, comments, tags, categories, images, tokens)
   - Defines repository interfaces (traits) that adapters must implement
   - No external dependencies except serialization and validation
   - Models include: User, Article, Comment, Tag, Category, Image, Token

2. **blog-app** - Application/Use Case layer
   - Contains application services (use cases) that orchestrate business logic
   - Services include: ArticleAppService, UserAppService, CommentAppService, CategoryAppService, TagAppService, TokenAppService, ImageAppService
   - Contains query services for complex read operations (articles_by_tag, tags_attached_article, article_image)
   - Depends on: blog-domain

3. **blog-adapter** - Infrastructure/Adapter layer
   - Implements repository interfaces defined in blog-domain
   - Database implementations using SQLx with PostgreSQL
   - Identity provider (IDP) integration for token management
   - Query service implementations for optimized read operations
   - Depends on: blog-domain, blog-app

4. **blog-driver** - Framework/Driver layer (entry point)
   - HTTP handlers using Axum web framework
   - Request/response models specific to HTTP API
   - Server configuration and dependency injection setup
   - CORS configuration with cookie-based authentication
   - Depends on: blog-domain, blog-app, blog-adapter

### Dependency Flow

```
blog-driver -> blog-adapter -> blog-app -> blog-domain
                      |                       ^
                      +----------------------+
```

The dependency flow ensures that domain logic remains independent of infrastructure concerns.

## Development Commands

### Running the Application

```bash
# Development mode with auto-reload
make dev

# Run with Docker Compose (includes nginx reverse proxy)
make run-local-container
```

### Building and Testing

```bash
# Build the entire workspace
cargo build

# Run all tests
cargo test

# Run tests for a specific package
cargo test -p blog-domain
cargo test -p blog-app
cargo test -p blog-adapter
cargo test -p blog-driver

# Run a specific test
cargo test test_name

# Build for release
cargo build --release
```

### Database

The project uses SQLx with PostgreSQL. Database connection is configured via the `DATABASE_URL` environment variable in `.env`.

Tests in blog-adapter that require database access use the `database-test` feature flag:
```bash
cargo test -p blog-adapter --features database-test
```

## Environment Configuration

Required environment variables (see `.env` file):
- `DATABASE_URL` - PostgreSQL connection string
- `INTERNAL_API_DOMAIN` - Server bind address (e.g., "0.0.0.0:8000")
- `CLIENT_ADDRS` - Comma-separated list of allowed CORS origins
- `MASTER_KEY` - Key for encrypted cookie management (generate with `openssl rand -base64 64`)

## Key Architectural Patterns

### Repository Pattern
- Repository traits are defined in blog-domain (e.g., `IUserRepository`)
- Concrete implementations live in blog-adapter (e.g., `UserRepository`)
- Repositories handle all data persistence operations

### Service Layer Pattern
- Domain services in blog-app contain business logic
- App services coordinate between repositories and domain services
- Query services handle complex read operations that join multiple entities

### Dependency Injection
- All dependencies are manually wired in `blog-driver/src/server/mod.rs`
- Services receive repository instances through constructors
- Single `PgPool` instance is shared across all repositories

## Docker Deployment

```bash
# Build Docker image for staging
make build-stg

# Push to GitHub Container Registry
make push-stg

# Build and deploy (requires GHCR_PAT and GHCR_USER env vars)
make deploy-stg
```

## Code Organization Notes

- Each domain entity (article, user, comment, etc.) has its own module across all layers
- Query services are separated from command services (CQRS-lite pattern)
- Authentication uses JWT tokens managed through the TokenAppService
- Image handling includes validation and encryption capabilities
- The driver layer contains cookie management for session handling
