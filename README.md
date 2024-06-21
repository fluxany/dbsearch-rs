# dbsearch
Application used to search databases using Retrieval Augmented Generation (RAG).

# Configuration
Contents of config.yml.
```yaml
agent_prompt: "...insert prompt..."
query: "Summarize this text."
```

# Usage
Build and run dbsearch.
```bash
cargo build
cargo run -- -c config.yml file.pdf
```

Create Redis Stack container.
```bash
docker run -d --name redis-stack -p 6379:6379 -p 8001:8001 redis/redis-stack:latest
```
