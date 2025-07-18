# RSSGen

## About

RSSGen is an intelligent RSS feed generator that automatically extracts and converts blog content into RSS feeds using AI-powered web scraping. The system analyzes blog website URL's that you pass, identifies post structures, and generates structured RSS feeds with or without manual configuration.

## Features

- **AI-Powered Content Analysis**: Uses Google Gemini AI to automatically identify blog post structures and content selectors
- **Supports Multiple RSS Formats**: Supports Atom RSS and RSS 2.0 formats
- **Automatic Feed Discovery**: Analyzes blog homepages to extract metadata and discover post links
- **Intelligent Web Scraping**: Dynamically extracts title and content from posts
- **Self-Host Support**: Containerized deployment with Docker Compose

## Current API Endpoints

### Feeds
- `GET /feeds` - List all feeds
- `POST /feeds` - Create a new feed from a blog URL
- `PATCH /feeds/{id}` - Update feed information
- `DELETE /feeds/{id}` - Delete a feed
- `GET /feeds/scrape` - Analyze a URL for feed metadata

### Posts
- `GET /feeds/{feed_id}/posts` - Get all posts for a feed
- `GET /feeds/{feed_id}/posts/scrape` - Scrape a single post for a feed

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Docker
- Google Gemini API key

### Environment Setup

Create a `.env` file in the `api` directory:

```env
DATABASE_URL=postgresql://username:password@localhost:5433/rssgen
GEMINI_API_KEY=your_gemini_api_key_here
POSTGRES_USER=your_username
POSTGRES_PASSWORD=your_password
POSTGRES_DB=rssgen
```

### Local Development

1. **Start the database**:
   ```bash
   cd api
   docker compose up -d
   ```

2. **Run database migrations**:
   ```bash
   diesel migration run
   ```

3. **Start the development server**:
   ```bash
   cargo run
   ```

The API will be available at `http://localhost:8080`.


## Database Schema

The application uses three main tables:

- **feeds**: Store blog feed information and metadata
- **posts**: Store individual blog posts with content
- **post_selectors**: Store CSS selectors for content extraction

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## Roadmap

- [ ] RSS feed generation and serving
- [ ] Scheduled automatic feed updates
- [ ] Support for multiple AI providers
- [ ] Feed validation and quality scoring
- [ ] Web interface for feed management
- [ ] Integration with popular RSS readers
