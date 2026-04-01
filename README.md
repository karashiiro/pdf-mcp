# pdf-mcp

An MCP server that extracts text and images from PDF documents using [pdf_oxide](https://crates.io/crates/pdf_oxide).

## Tool

### `extract_pdf`

Extracts text and images from a PDF document.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `pdf_base64` | `string` | The PDF file content, encoded as a base64 string |

**Returns** zero or more content blocks:
- **Text blocks** — extracted text per page (empty pages are skipped)
- **Image blocks** — embedded images as base64-encoded PNGs

## Usage

### Building

```sh
cargo build --release
```

### Claude Code

Add to your Claude Code MCP settings:

```json
{
  "mcpServers": {
    "pdf": {
      "command": "/path/to/pdf-mcp"
    }
  }
}
```

### Claude Desktop

Add to your Claude Desktop config:

```json
{
  "mcpServers": {
    "pdf": {
      "command": "/path/to/pdf-mcp"
    }
  }
}
```
