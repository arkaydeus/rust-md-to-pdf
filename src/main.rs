use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Result};
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
    markdown_to_html(markdown, &options)
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

    // Run wkhtmltopdf
    let output = Command::new("wkhtmltopdf")
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
        // Configure CORS middleware
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .route("/health", web::get().to(health_check))
            .route("/convert", web::post().to(convert_markdown_to_pdf))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
