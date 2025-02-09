# Rust Markdown to PDF Converter

A REST API service that converts Markdown text to PDF documents.

## Prerequisites

1. Install Rust (if not already installed):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install wkhtmltopdf (required for PDF generation):
   - On macOS:
     ```bash
     brew install wkhtmltopdf
     ```
   - On Ubuntu/Debian:
     ```bash
     sudo apt-get install wkhtmltopdf
     ```
   - On Windows:
     Download and install from [wkhtmltopdf downloads](https://wkhtmltopdf.org/downloads.html)

## Building and Running

### Local Development

1. Clone the repository
2. Build and run the project:
   ```bash
   cargo run
   ```
   The server will start at `http://0.0.0.0:8080`

### Docker

1. Build and run using Docker Compose:
   ```bash
   docker compose up --build
   ```
   The server will be available at `http://localhost:8080`

## API Usage

All endpoints support CORS and allow:

- Any origin (\*)
- Any HTTP method
- Any headers
- Preflight cache of 1 hour (3600 seconds)

### Health Check

**Endpoint:** `GET /health`

**Response:**

```json
{
  "status": "healthy",
  "version": "1.0.0"
}
```

The health check verifies that the service and its dependencies (wkhtmltopdf) are working properly.
Possible status responses:

- `200 OK` with "healthy" status if everything is working
- `503 Service Unavailable` with "unhealthy" status if wkhtmltopdf is not available

### Convert Markdown to PDF

**Endpoint:** `POST /convert`

**Request Body:**

```json
{
  "markdown": "# Your Markdown Text\n\nThis is a paragraph."
}
```

**Response:**

- Content-Type: application/pdf
- Content-Disposition: attachment; filename="document.pdf"
- Body: Binary PDF data

**Example using curl:**

```bash
# Basic usage - saves output as document.pdf
curl -X POST http://localhost:8080/convert \
  -H "Content-Type: application/json" \
  -d '{"markdown": "# Hello World\n\nThis is a test."}' \
  --output document.pdf

# Convert README.md to PDF
curl -X POST http://localhost:8080/convert \
  -H "Content-Type: application/json" \
  -d "{\"markdown\": $(cat README.md | jq -Rs .)}" \
  --output readme.pdf
```

## Error Handling

The API will return:

- `200 OK` with the PDF data on success
- `500 Internal Server Error` if PDF generation fails

## License

MIT
