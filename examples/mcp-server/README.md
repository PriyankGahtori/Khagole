# Khagole Model Context Protocol (MCP) Server for AI Agents

Enables AI Agents (**Claude Desktop**, **Cursor**, **Antigravity**, **LangChain**, **CrewAI**, **AutoGen**, and web chat agents) to perform high-precision Vedic Jyotish and NASA JPL ephemerides calculations via standardized Model Context Protocol (MCP) tools.

---

## 🌟 Supported Hosting & Execution Modes

| Mode | Transport | Typical Clients | Command |
|---|---|---|---|
| **Hosted Remote Server** | **SSE (HTTP)** | Cloud Agents, LangChain, Flowise, AutoGen, CrewAI, Remote Web Apps | `python server.py --transport sse --port 8000` |
| **Local Desktop Server** | **stdio** | Claude Desktop, Cursor IDE, Windsurf, Antigravity, Cline | `python server.py --transport stdio` |
| **Docker Cloud Container** | **SSE (HTTP)** | Fly.io, Railway, Google Cloud Run, AWS ECS, Render | `docker run -p 8000:8000 khagole-mcp` |

---

## 1. Hosting as a Remote Server (SSE over HTTP)

When hosted over HTTP/SSE, any AI Agent anywhere on your network or across the internet can connect and use Khagole tools.

### Running Locally / On a VM:
```bash
cd examples/mcp-server
pip install -r requirements.txt
python server.py --transport sse --host 0.0.0.0 --port 8000
```

The server exposes:
- **SSE Stream**: `http://localhost:8000/sse`
- **Messages Endpoint**: `http://localhost:8000/messages`
- **Health Check & Service Info**: `http://localhost:8000/` and `http://localhost:8000/health`

### 1-Click Docker Deployment:
Build and run the containerized MCP server with NASA JPL kernels included:

```bash
# Build Docker image
docker build -t khagole-mcp -f examples/mcp-server/Dockerfile .

# Run on port 8000
docker run -d -p 8000:8000 --name khagole-mcp khagole-mcp
```

### Deploying to Cloud Providers:
- **Google Cloud Run**: `gcloud run deploy khagole-mcp --source . --port 8000 --allow-unauthenticated`
- **Fly.io**: `fly launch && fly deploy`
- **Railway / Render**: Connect repository and select `examples/mcp-server/Dockerfile`.

---

## 2. Using with Local AI Desktop Apps (stdio)

### Claude Desktop
Add to your `claude_desktop_config.json` (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS or `%APPDATA%\Claude\claude_desktop_config.json` on Windows):

```json
{
  "mcpServers": {
    "khagole": {
      "command": "python",
      "args": [
        "/path/to/khagole/examples/mcp-server/server.py",
        "--transport",
        "stdio"
      ]
    }
  }
}
```

*Or use the native compiled Rust MCP server binary:*
```json
{
  "mcpServers": {
    "khagole": {
      "command": "cargo",
      "args": ["run", "--release", "--manifest-path", "/path/to/khagole/crates/khagole-mcp/Cargo.toml"]
    }
  }
}
```

### Cursor IDE
In Cursor Settings -> **Features** -> **MCP**:
- **Name**: `khagole`
- **Type**: `command`
- **Command**: `python /path/to/khagole/examples/mcp-server/server.py --transport stdio`

---

## 3. Connecting from AI Agent Code (Python / LangChain)

Any Python agent can connect to your hosted MCP server over SSE using the official `mcp` SDK:

```python
import asyncio
from mcp import ClientSession
from mcp.client.sse import sse_client

async def run_agent():
    # Connect to hosted MCP server over SSE
    async with sse_client("http://localhost:8000/sse") as (read_stream, write_stream):
        async with ClientSession(read_stream, write_stream) as session:
            await session.initialize()
            
            # List available tools
            tools = await session.list_tools()
            print("Available Tools:", [t.name for t in tools.tools])
            
            # Call live calculation
            result = await session.call_tool("calculate_chart", {
                "utc": "1955-02-25T03:15:00Z",
                "latitude": 37.7749,
                "longitude": -122.4194,
            })
            print("Chart Output:", result.content[0].text)

asyncio.run(run_agent())
```

---

## 4. Available MCP Tools

| Tool | Parameters | Description |
|---|---|---|
| `calculate_chart` | `utc`, `latitude`, `longitude`, `house_system`, `zodiac_type`, `ayanamsa`, `position_mode`, `node_type` | Computes high-precision Sidereal Vedic birth chart positions (Navagrahas + Lagna, DMS, House, Nakshatra, Pada 1–4, Lord, Motion). |
| `calculate_panchanga` | `sun_longitude`, `moon_longitude`, `utc`, `jd` | Computes Five Limbs of Vedic Time (Tithi, Vara, Nakshatra, Yoga, Karana, Lunar Month, Ritu, Ayana, Vikram/Shaka Samvat). |
| `calculate_divisional_chart` | `varga` (D1–D60), `utc`, `latitude`, `longitude` | Maps planetary coordinates into Vedic divisional charts (D9 Navamsha, D10 Dashamsha, etc.). |
| `calculate_vimshottari_dasha` | `moon_longitude`, `birth_utc`, `depth`, `year_type` | Computes the 120-year Vimshottari Dasha sequence tree using True Sidereal Solar Year (TSSY). |
| `get_celebrity_chart` | `celebrity_id` (`jobs`, `modi`, `einstein`, `sagan`) | Instantly generates a full horoscope, Panchanga, D9 Navamsha, and Vimshottari Dasha report for famous figures. |

---

## 5. Verification

The hosted MCP server is verified with automated tests covering both SSE stream transport and direct tool execution:

```bash
# Run the automated SSE agent client test:
python scratch/test_mcp_sse_client.py
```
