use actix_cors::Cors;
use actix_web::{http, web, App, HttpResponse, HttpServer, Result};
use anyhow::Context;
use comrak::{markdown_to_html, ComrakOptions};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct MarkdownRequest {
    markdown: String,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

/// Converts markdown text to HTML using comrak
fn markdown_to_html_converter(markdown: &str) -> String {
    let options = ComrakOptions::default();
    let content = markdown_to_html(markdown, &options);

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Document</title>
    <style>
        @page {{
            size: A4;
            margin: 10mm;
        }}
        html {{
            font-size: 16pt !important;
            width: 210mm;  /* A4 width */
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            line-height: 1.6;
            padding: 0 5em;
            font-size: 1rem !important;
            width: 100%;
            margin: 0;
            overflow-wrap: break-word;
            word-wrap: break-word;
            word-break: break-word;
        }}
        /* Force consistent sizes */
        p, div, span, li, td {{
            font-size: 1rem !important;
        }}
        h1 {{ font-size: 1.4rem !important; }}
        h2 {{ font-size: 1.2rem !important; }}
        h3 {{ font-size: 1.1rem !important; }}
        h4, h5, h6 {{ font-size: 1.1rem !important; }}
        /* Handle long URLs */
        a {{
            word-wrap: break-word;
            word-break: break-all;
            white-space: pre-wrap;
            overflow-wrap: break-word;
            max-width: 100%;
            display: inline-block;
        }}
    </style>
</head>
<body>
    {}
</body>
</html>"#,
        content
    )
}

/// Creates a temporary file with the given content and returns its path
fn create_temp_file(content: &str, extension: &str) -> anyhow::Result<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let file_name = format!("{}.{}", Uuid::new_v4(), extension);
    let file_path = temp_dir.join(file_name);

    fs::write(&file_path, content)?;
    Ok(file_path)
}

/// Converts HTML to PDF using wkhtmltopdf command line tool
async fn html_to_pdf(html: &str) -> anyhow::Result<Vec<u8>> {
    // Create temporary HTML file
    let html_path =
        create_temp_file(html, "html").context("Failed to create temporary HTML file")?;

    // Create temporary PDF file path
    let pdf_path = html_path.with_extension("pdf");

    // Run wkhtmltopdf with margin settings
    let output = Command::new("wkhtmltopdf")
        .arg("--page-size")
        .arg("A4")
        .arg("--dpi")
        .arg("96")
        .arg("--margin-top")
        .arg("20mm")
        .arg("--margin-bottom")
        .arg("20mm")
        .arg("--disable-smart-shrinking")
        .arg("--enable-local-file-access")
        .arg("--zoom")
        .arg("1.0")
        .arg("--print-media-type")
        .arg("--no-background")
        .arg(&html_path)
        .arg(&pdf_path)
        .output()
        .context("Failed to execute wkhtmltopdf")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "wkhtmltopdf failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Read the generated PDF
    let pdf_content = fs::read(&pdf_path).context("Failed to read generated PDF")?;

    // Clean up temporary files
    let _ = fs::remove_file(html_path);
    let _ = fs::remove_file(pdf_path);

    Ok(pdf_content)
}

/// Handles the POST request to convert markdown to PDF
async fn convert_markdown_to_pdf(payload: web::Json<MarkdownRequest>) -> Result<HttpResponse> {
    // Convert markdown to HTML
    let html = markdown_to_html_converter(&payload.markdown);

    // Convert HTML to PDF
    match html_to_pdf(&html).await {
        Ok(pdf_bytes) => Ok(HttpResponse::Ok()
            .content_type("application/pdf")
            .append_header((
                "Content-Disposition",
                "attachment; filename=\"document.pdf\"",
            ))
            .body(pdf_bytes)),
        Err(e) => {
            eprintln!("Error converting to PDF: {}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

/// Health check endpoint that verifies the service and its dependencies are working
async fn health_check() -> Result<HttpResponse> {
    // Check if wkhtmltopdf is available
    match Command::new("wkhtmltopdf").arg("--version").output() {
        Ok(_) => Ok(HttpResponse::Ok().json(HealthResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        })),
        Err(_) => Ok(HttpResponse::ServiceUnavailable().json(HealthResponse {
            status: "unhealthy - wkhtmltopdf not found".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        })),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Check if wkhtmltopdf is installed
    if let Err(_) = Command::new("wkhtmltopdf").arg("--version").output() {
        eprintln!("Error: wkhtmltopdf is not installed. Please install it first.");
        std::process::exit(1);
    }

    println!(
        "Starting rust-md-to-pdf v{} at http://0.0.0.0:8080",
        env!("CARGO_PKG_VERSION")
    );

    HttpServer::new(|| {
        // Configure CORS middleware with permissive settings
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .route("/health", web::get().to(health_check))
            .route("/convert", web::post().to(convert_markdown_to_pdf))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
