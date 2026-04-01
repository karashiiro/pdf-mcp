use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use pdf_oxide::PdfDocument;
use pdf_oxide::extractors::PdfImage;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, Implementation, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;

const SERVER_INSTRUCTIONS: &str = "\
This server extracts content from PDF documents. \
Provide a base64-encoded PDF to the extract_pdf tool and receive \
the extracted text and images as content blocks.";

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExtractPdfParams {
    #[schemars(description = "The PDF file content, encoded as a base64 string")]
    pub pdf_base64: String,
}

#[derive(Debug, Clone)]
pub struct PdfServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl PdfServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        name = "extract_pdf",
        description = "Extract text and images from a PDF document. Takes a base64-encoded PDF and returns the extracted content as text and image blocks."
    )]
    fn extract_pdf(
        &self,
        Parameters(ExtractPdfParams { pdf_base64 }): Parameters<ExtractPdfParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let pdf_bytes = BASE64
            .decode(&pdf_base64)
            .map_err(|e| rmcp::ErrorData::invalid_params(format!("Failed to decode base64: {e}"), None))?;

        let mut doc = PdfDocument::from_bytes(pdf_bytes)
            .map_err(|e| rmcp::ErrorData::invalid_params(format!("Failed to parse PDF: {e}"), None))?;

        let page_count = doc
            .page_count()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to get page count: {e}"), None))?;

        let mut contents: Vec<Content> = Vec::new();

        for page_idx in 0..page_count {
            match doc.extract_text(page_idx) {
                Ok(text) if !text.trim().is_empty() => {
                    contents.push(Content::text(text));
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!(page = page_idx, error = %e, "Failed to extract text");
                }
            }

            match doc.extract_images(page_idx) {
                Ok(images) => {
                    for image in &images {
                        match image_to_png_base64(image) {
                            Ok(png_b64) => {
                                contents.push(Content::image(png_b64, "image/png"));
                            }
                            Err(e) => {
                                tracing::warn!(
                                    page = page_idx,
                                    error = %e,
                                    "Failed to encode image"
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(page = page_idx, error = %e, "Failed to extract images");
                }
            }
        }

        Ok(CallToolResult::success(contents))
    }
}

fn image_to_png_base64(image: &PdfImage) -> anyhow::Result<String> {
    let png_bytes = image
        .to_png_bytes()
        .map_err(|e| anyhow::anyhow!("PNG encoding failed: {e}"))?;
    Ok(BASE64.encode(&png_bytes))
}

#[tool_handler]
impl ServerHandler for PdfServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(SERVER_INSTRUCTIONS.to_string())
    }
}
