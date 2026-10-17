# Khagole Hosted MCP Server Dockerfile
# Provides high-precision Vedic Jyotish & NASA JPL Ephemerides over SSE (HTTP) & stdio for AI Agents

FROM python:3.11-slim

WORKDIR /app

# Install system dependencies & curl
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install python dependencies
COPY examples/mcp-server/requirements.txt /app/requirements.txt
RUN pip install --no-cache-dir -r requirements.txt

# Download NASA JPL DE440s ephemeris kernel (covers 1849 to 2150)
RUN mkdir -p /app/kernels && \
    curl -fL -o /app/kernels/de440s.bsp \
    "https://raw.githubusercontent.com/PriyankGahtori/Khagole/main/kernels/de440s.bsp" || \
    curl -fL -o /app/kernels/de440s.bsp \
    "https://priyankgahtori.github.io/Khagole/ephemeris/de440s-1920-2070.bsp"

# Copy MCP server code
COPY examples/mcp-server/server.py /app/server.py

ENV KHAGOLE_KERNEL_PATH=/app/kernels/de440s.bsp
ENV PYTHONUNBUFFERED=1
EXPOSE 8000

# Default: Hosted SSE transport on port 8000
ENTRYPOINT ["python", "server.py"]
CMD ["--transport", "sse", "--host", "0.0.0.0", "--port", "8000"]
