# GAP-009 — Make nginx upstream configurable (resolved)

**Resolved 2026-09-08.** The web image renders its nginx configuration at
container startup. Set `API_UPSTREAM` to the complete API URL reachable from
the web container; the bundled Compose and Swarm deployments use
`http://server:8456`.

## Original problem

`apps/web/nginx.conf` hardcodes the upstream as:

```
proxy_pass http://zealot_zealot:8456/;
```

`zealot_zealot` is Docker Compose's internal DNS name for a service named `zealot`
in a project named `zealot`. If either the project name or the service name changes,
the web image silently breaks — every API request returns a 502, because nginx cannot
resolve the upstream hostname.

This is documented as a known limitation:

> "The nginx upstream is hardcoded. `apps/web/nginx.conf` references
> `zealot_zealot:8456`. If you change the Compose project name or service name, the
> web image must be rebuilt with an updated `nginx.conf`."

The limitation surfaces when:
- An operator runs with `-p my-project` or sets `COMPOSE_PROJECT_NAME`
- Someone forks the repo and changes the compose service name
- A Docker Swarm or Kubernetes deployment uses a different service name

## Implemented design

Use nginx's built-in runtime template rendering:

1. The `nginx.conf.template` proxy target is:
   ```nginx
   proxy_pass ${API_UPSTREAM}/;
   ```

2. The official nginx entrypoint renders
   `/etc/nginx/templates/default.conf.template` into its active configuration.

3. `API_UPSTREAM` defaults to `http://server:8456` in the image and is set
   explicitly in the bundled Compose files.

## Files to change

- `apps/web/nginx.conf.template` — uses `${API_UPSTREAM}`
- `apps/web/Dockerfile` — installs the template and declares its default
- `scripts/zealot-compose.yml` and `docker-compose.yml` — set `API_UPSTREAM`
- `docs/deployment.md` and `docs/troubleshooting.md` — document the variable
